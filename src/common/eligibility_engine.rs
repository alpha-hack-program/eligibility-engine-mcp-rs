use serde::{Deserialize, Serialize, Deserializer, de::Error as DeError};
use zen_engine::DecisionEngine;
use zen_engine::model::DecisionContent;
use zen_engine::{EvaluationError, NodeError};
use std::fmt;

use super::metrics::{increment_requests, increment_errors, RequestTimer};

use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo, CallToolResult, Content},
    ErrorData as McpError,
    schemars, tool, tool_handler, tool_router,
};

// =================== ERROR STRUCTURES ===================

#[derive(Debug, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidationErrorSource {
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Deserialize)]
pub struct ValidationErrorDetails {
    pub source: ValidationErrorSource,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub error_type: String,
}

#[derive(Debug)]
pub enum UnpaidLeaveError {
    ValidationError(Vec<ValidationError>),
    ZenEngineError(EvaluationError),
    SerializationError(serde_json::Error),
}

impl fmt::Display for UnpaidLeaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnpaidLeaveError::ValidationError(errors) => {
                write!(f, "Validation errors:\n")?;
                for error in errors {
                    write!(f, "  - {}: {}\n", error.path, error.message)?;
                }
                Ok(())
            },
            UnpaidLeaveError::ZenEngineError(e) => write!(f, "Decision engine error: {}", e),
            UnpaidLeaveError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for UnpaidLeaveError {}

impl From<EvaluationError> for UnpaidLeaveError {
    fn from(error: EvaluationError) -> Self {
        UnpaidLeaveError::ZenEngineError(error)
    }
}

impl From<serde_json::Error> for UnpaidLeaveError {
    fn from(error: serde_json::Error) -> Self {
        UnpaidLeaveError::SerializationError(error)
    }
}

// =================== AUXILIARY FUNCTIONS ===================

/// Deserializes a value that can be bool or string ("true"/"false")
fn deserialize_bool_or_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Visitor;
    use std::fmt;

    struct BoolOrStringVisitor;

    impl<'de> Visitor<'de> for BoolOrStringVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("bool or string")
        }

        fn visit_bool<E>(self, value: bool) -> Result<bool, E>
        where
            E: DeError,
        {
            Ok(value)
        }

        fn visit_str<E>(self, value: &str) -> Result<bool, E>
        where
            E: DeError,
        {
            match value.to_lowercase().as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(DeError::custom(format!("invalid boolean string: {}", value))),
            }
        }

        fn visit_string<E>(self, value: String) -> Result<bool, E>
        where
            E: DeError,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(BoolOrStringVisitor)
}

/// Deserializes a value that can be f64 or string
fn deserialize_f64_or_string<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Visitor;
    use std::fmt;

    struct F64OrStringVisitor;

    impl<'de> Visitor<'de> for F64OrStringVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("f64, string, or null")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Option<f64>, E>
        where
            E: DeError,
        {
            Ok(Some(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<f64>, E>
        where
            E: DeError,
        {
            Ok(Some(value as f64))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<f64>, E>
        where
            E: DeError,
        {
            Ok(Some(value as f64))
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<f64>, E>
        where
            E: DeError,
        {
            value.parse::<f64>()
                .map(Some)
                .map_err(|_| DeError::custom(format!("invalid number string: {}", value)))
        }

        fn visit_string<E>(self, value: String) -> Result<Option<f64>, E>
        where
            E: DeError,
        {
            self.visit_str(&value)
        }

        fn visit_none<E>(self) -> Result<Option<f64>, E>
        where
            E: DeError,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Option<f64>, E>
        where
            E: DeError,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(F64OrStringVisitor)
}

// =================== DATA STRUCTURES ===================

// Direct parameters structure for MCP (flattened)
#[derive(Debug, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct UnpaidLeaveDirectParams {
    #[schemars(description = "Family relationship with the person who needs care. VALID VALUES: 'father', 'mother', 'parent', 'son', 'daughter', 'spouse', 'partner', 'husband', 'wife', 'foster_parent'. Example: My mother had an accident and I'm taking care of her => 'son'; I had a baby => 'mother' or 'parent'")]
    pub relationship: String,
    
    #[schemars(description = "Situation that motivates the need for care. VALID VALUES: 'birth', 'adoption', 'foster_care', 'multiple_birth', 'multiple_adoption', 'multiple_foster_care', 'illness', 'accident'. If number of children born or adopted or fostered is greater than one at the same time, USE 'multiple_birth' or 'multiple_adoption' or 'multiple_foster_care'. Example: I had a baby => 'birth'; I adopted a child => 'adoption'; I'm fostering two kids => 'multiple_foster_care'")]
    pub situation: String,
    
    #[schemars(description = "Are you a single parent? Only relevant for birth/adoption situations, otherwise it is not relevant and should be always false")]
    #[serde(deserialize_with = "deserialize_bool_or_string")]
    pub is_single_parent: bool,
    
    #[schemars(description = "Total number of children you'll have after birth/adoption (0 for illness/accident care)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_f64_or_string")]
    pub total_children_after: Option<f64>,
}

// Internal structure for the ZEN engine (nested)
#[derive(Debug, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct UnpaidLeaveInput {
    #[schemars(description = "Family relationship with the person who needs care. VALID VALUES: 'father', 'mother', 'parent', 'son', 'daughter', 'spouse', 'partner', 'husband', 'wife', 'foster_parent'. Example: My mother had an accident and I'm taking care of her => 'son'; I had a baby => 'mother' or 'parent'")]
    pub relationship: String,
    
    #[schemars(description = "Situation that motivates the need for care. VALID VALUES: 'birth', 'adoption', 'foster_care', 'multiple_birth', 'multiple_adoption', 'multiple_foster_care', 'illness', 'accident'. If number of children born or adopted or fostered is greater than one at the same time, USE 'multiple_birth' or 'multiple_adoption' or 'multiple_foster_care'. Example: I had a baby => 'birth'; I adopted a child => 'adoption'; I'm fostering two kids => 'multiple_foster_care'")]
    pub situation: String,
    
    #[schemars(description = "Are you a single parent? Only relevant for birth/adoption situations, otherwise it is not relevant and should be always false")]
    pub is_single_parent: bool,
    
    #[schemars(description = "Total number of children you'll have after birth/adoption (0 for illness/accident care)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_children_after: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UnpaidLeaveRequest {
    #[schemars(description = "Input data to evaluate unpaid leave assistance eligibility")]
    pub input: UnpaidLeaveInput,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct UnpaidLeaveResponse {
    #[schemars(description = "Evaluation result")]
    pub output: UnpaidLeaveOutputForSchema,
    #[serde(default)]
    pub input: Option<UnpaidLeaveInput>,
    #[serde(default)]
    pub relationship_valid: Option<bool>,
}

// Estructura para el schema JSON (para documentación MCP)
#[derive(Debug, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct UnpaidLeaveOutputForSchema {
    #[schemars(description = "Description of the applicable case")]
    pub description: String,
    
    #[schemars(description = "Monthly benefit amount in euros. 725€ for Case A (family care), 500€ for other valid cases, 0€ if not eligible")]
    pub monthly_benefit: i32,
    
    #[schemars(description = "Detailed description of additional requirements that must be met")]
    #[serde(default)]
    pub additional_requirements: String,
    
    #[schemars(description = "Letter of the applicable case according to regulations (A, B, C, D, E) or empty if not eligible")]
    pub case: String,
    
    #[schemars(description = "Does it meet the intrinsic requirements to potentially be entitled to the benefit?")]
    pub potentially_eligible: bool,
    
    #[schemars(description = "List of errors or unmet requirements")]
    #[serde(default)]
    pub errores: Vec<String>,
    
    #[schemars(description = "List of warnings or additional relevant information")]
    #[serde(default)]
    pub warnings: Vec<String>,
}

// =================== DECISION ENGINE ===================

#[derive(Debug, Clone)]
struct UnpaidLeaveDecisionEngine;

impl UnpaidLeaveDecisionEngine {
    fn new() -> Self {
        Self
    }

    async fn evaluate_unpaid_leave(&self, request: &UnpaidLeaveRequest) -> Result<UnpaidLeaveResponse, UnpaidLeaveError> {
        // Load the decision from the JSON file
        let decision_content: DecisionContent = 
            serde_json::from_str(include_str!("unpaid-leave-assistance-2025.json"))
            .map_err(UnpaidLeaveError::from)?;
        let engine = DecisionEngine::default();
        let decision = engine.create_decision(decision_content.into());
        
        // Convert struct to JSON and then to Variable
        let json_value = serde_json::to_value(request)?;
        
        match decision.evaluate(json_value.into()).await {
            Ok(result) => {
                // Convert result from Variable to Value and then deserialize directly
                let result_value: serde_json::Value = result.result.into();
                let response: UnpaidLeaveResponse = serde_json::from_value(result_value)?;
                
                Ok(response)
            },
            Err(zen_error) => {
                // Attempt to extract validation error information
                if let Some(validation_errors) = Self::extract_validation_errors(&zen_error) {
                    Err(UnpaidLeaveError::ValidationError(validation_errors))
                } else {
                    Err(UnpaidLeaveError::ZenEngineError(*zen_error))
                }
            }
        }
    }
    
    // Helper function to extract validation errors from ZEN error
    fn extract_validation_errors(error: &EvaluationError) -> Option<Vec<ValidationError>> {
        if let EvaluationError::NodeError(node_error) = error {
            if let Some(errors) = Self::extract_from_node_error(node_error) {
                return Some(errors);
            }
        }
        
        let error_str = format!("{:?}", error);
        Self::extract_from_error_string(&error_str)
    }
    
    fn extract_from_node_error(node_error: &NodeError) -> Option<Vec<ValidationError>> {
        let source_str = format!("{:?}", node_error.source);
        Self::extract_json_from_string(&source_str)
    }
    
    fn extract_from_error_string(error_str: &str) -> Option<Vec<ValidationError>> {
        Self::extract_json_from_string(error_str)
    }
    
    fn extract_json_from_string(text: &str) -> Option<Vec<ValidationError>> {
        let patterns = vec![
            (r#"{"source":{"errors":"#, r#""type":"Validation"}"#),
            (r#"{"errors":"#, r#""type":"Validation"}"#),
            (r#""errors":["#, r#"]"#),
        ];
        
        for (start_pattern, end_pattern) in patterns {
            if let Some(start) = text.find(start_pattern) {
                let search_from = start + start_pattern.len();
                if let Some(relative_end) = text[search_from..].find(end_pattern) {
                    let end = search_from + relative_end + end_pattern.len();
                    let json_candidate = &text[start..end];
                    
                    if let Ok(details) = serde_json::from_str::<ValidationErrorDetails>(json_candidate) {
                        return Some(details.source.errors);
                    }
                    
                    if let Some(errors) = Self::manual_extract_errors(text) {
                        return Some(errors);
                    }
                }
            }
        }
        
        Self::manual_extract_errors(text)
    }
    
    fn manual_extract_errors(text: &str) -> Option<Vec<ValidationError>> {
        if text.contains("is not one of") {
            let lines: Vec<&str> = text.split(',').collect();
            
            let mut message = String::new();
            let mut path = String::new();
            
            for line in lines {
                if line.contains("\"message\":") {
                    if let Some(start) = line.find("\"message\":\"") {
                        let msg_start = start + "\"message\":\"".len();
                        if let Some(end) = line[msg_start..].find("\"") {
                            message = line[msg_start..msg_start + end].to_string();
                        }
                    }
                }
                if line.contains("\"path\":") {
                    if let Some(start) = line.find("\"path\":\"") {
                        let path_start = start + "\"path\":\"".len();
                        if let Some(end) = line[path_start..].find("\"") {
                            path = line[path_start..path_start + end].to_string();
                        }
                    }
                }
            }
            
            if !message.is_empty() {
                if path.is_empty() {
                    path = "/input/unknown".to_string();
                }
                return Some(vec![ValidationError { message, path }]);
            }
        }
        
        None
    }
}

// =================== Eligibility ENGINE MCP ===================

#[derive(Debug, Clone)]
pub struct EligibilityEngine {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl EligibilityEngine {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Evaluates unpaid leave assistance eligibility according to fictional regulations
    /// 
    /// IMPORTANT: Use the exact values specified in each parameter.
    /// IMPORTANT: If number of children is greater than one, USE 'multiple_birth' or 'multiple_adoption' or 'multiple_foster_care'.
    /// IMPORTANT: If no information regarding the family structure use always false.
    /// IMPORTANT: If no information regarding the number of children use always 0.
    #[tool(description = "Evaluates unpaid leave assistance eligibility according to legal regulations. Determines case (A-E) and amount (0€/500€/725€). CASES: A=Sick family care (725€), B=Third child+ (500€), C=Adoption (500€), D=Multiple (500€), E=Single-parent (500€). USE EXACT VALUES: relationship ('father'/'mother'/'parent'/'son'/'daughter'/'spouse'/'partner'/'husband'/'wife'/'foster_parent'), situation ('birth'/'adoption'/'foster_care'/'multiple_birth'/'multiple_adoption'/'multiple_foster_care'/'illness'/'accident'), is_single_parent (true/false), total_children_after (number).")]
    pub async fn evaluate_unpaid_leave_eligibility(
        &self, 
        Parameters(direct_params): Parameters<UnpaidLeaveDirectParams>
    ) -> Result<CallToolResult, McpError> {
        // Initialize metrics tracking
        let _timer = RequestTimer::new();
        increment_requests();
        // Convert direct parameters to nested structure expected by the engine
        let request = UnpaidLeaveRequest {
            input: UnpaidLeaveInput {
                relationship: direct_params.relationship,
                situation: direct_params.situation,
                is_single_parent: direct_params.is_single_parent,
                total_children_after: direct_params.total_children_after,
            }
        };

        // Use tokio::task::spawn_blocking for operations that are not Send
        let result = tokio::task::spawn_blocking(move || {
            // Create a tokio runtime for the async operation inside the blocking block
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let engine = UnpaidLeaveDecisionEngine::new();
                engine.evaluate_unpaid_leave(&request).await
            })
        }).await;
        
        match result {
            Ok(eval_result) => {
                match eval_result {
                    Ok(response) => {
                        // Serialize the response to JSON and return as success
                        match serde_json::to_string_pretty(&response) {
                            Ok(json_str) => Ok(CallToolResult::success(vec![Content::text(json_str)])),
                            Err(e) => {
                                increment_errors();
                                Ok(CallToolResult::error(vec![Content::text(format!(
                                    "Error serializing response: {}", e
                                ))]))
                            }
                        }
                    },
                    Err(e) => {
                        increment_errors();
                        let error_msg = match e {
                            UnpaidLeaveError::ValidationError(validation_errors) => {
                                let mut msg = "Validation errors:\n".to_string();
                                for error in validation_errors {
                                    msg.push_str(&format!("  - Field '{}': {}\n", error.path, error.message));
                                }
                                msg
                            },
                            _ => format!("Evaluation error: {}", e)
                        };
                        Ok(CallToolResult::error(vec![Content::text(error_msg)]))
                    }
                }
            },
            Err(join_error) => {
                increment_errors();
                Ok(CallToolResult::error(vec![Content::text(format!(
                    "Internal error: {}", join_error
                ))]))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for EligibilityEngine {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Eligibility Engine for leave assistance according to legal regulations. \
                 \n\n** IMPORTANT TOOL USAGE INSTRUCTIONS **\
                 \n\n1. ALWAYS use the EXACT values specified for each parameter, CASE SENSITIVE\
                 \n\n2. For relationship, use ONLY: 'father', 'mother', 'parent', 'son', 'daughter', 'spouse', 'partner', 'husband', 'wife', 'foster_parent'\
                 \n\n3. For situation, use ONLY: 'birth', 'adoption', 'foster_care', 'multiple_birth', 'multiple_adoption', 'multiple_foster_care', 'illness', 'accident'. If number of children is greater than one, USE 'multiple_birth' or 'multiple_adoption' or 'multiple_foster_care'\
                 \n\n4. For is_single_parent, use ONLY: true (for single-parent families) or false (for families with both parents). If no information regarding the family structure use always false\
                 \n\n5. For total_children_after, use whole numbers (eg: 1, 2, 3, 4, 5). ONLY if situation is 'birth' or 'adoption' or 'foster_care' or 'multiple_birth' or 'multiple_adoption' or 'multiple_foster_care'
                 \n\nCORRECT USAGE EXAMPLES:\
                 \n• Single father with baby: relationship='father', situation='birth', is_single_parent=true, total_children_after=1\
                 \n• Son caring for sick father: relationship='father', situation='illness', is_single_parent=false, total_children_after=0\
                 \n• Family with third child: relationship='mother', situation='birth', is_single_parent=false, total_children_after=3\
                 \n• Family with multiple children: relationship='mother', situation='multiple_birth', is_single_parent=false, total_children_after=3\
                 \n• Family with multiple children: relationship='mother', situation='multiple_adoption', is_single_parent=false, total_children_after=3\
                 \n• Family with multiple children: relationship='mother', situation='multiple_foster_care', is_single_parent=false, total_children_after=3\
                 \n\nCASES EVALUATED:\
                 \nA) Sick/injured family care (725€/month)\
                 \nB) Third child+ with newborn (500€/month)\
                 \nC) Adoption/foster care (500€/month)\
                 \nD) Multiple births/adoptions (500€/month)\
                 \nE) Single-parent families (500€/month)".into()
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: rmcp::model::Implementation {
                name:"eligibility-engine".to_string(),
                version:"1.0.0".to_string(), 
                title: None, 
                icons: None, 
                website_url: None 
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_eligibility_engine_case_a() {
        let eligibility_engine = EligibilityEngine::new();
        let direct_params = UnpaidLeaveDirectParams {
            relationship: "mother".to_string(),
            situation: "illness".to_string(),
            is_single_parent: false,
            total_children_after: None,
        };
        
        let result = eligibility_engine.evaluate_unpaid_leave_eligibility(Parameters(direct_params)).await;
        match result {
            Ok(call_result) => {
                // Check if it's a success result
                println!("Resultado Supuesto A: {:?}", call_result);
                let content = call_result.content;
                assert!(!content.is_empty(), "Content should not be empty");
                let raw_content = &content[0].raw;
                // Extract the text from the raw content, it has to be a string
                let json_text = &raw_content.as_text().unwrap().text;
                let response: UnpaidLeaveResponse = serde_json::from_str(json_text).unwrap();
                assert_eq!(response.output.case, "A");
                assert!(response.output.potentially_eligible);
                assert_eq!(response.output.monthly_benefit, 725);
                
            },
            Err(e) => panic!("Error inesperado: {}", e),
        }
    }

    #[tokio::test] 
    async fn test_eligibility_engine_case_e() {
        let eligibility_engine = EligibilityEngine::new();
        let direct_params = UnpaidLeaveDirectParams {
            relationship: "mother".to_string(),
            situation: "birth".to_string(),
            is_single_parent: true,
            total_children_after: Some(1.0),
        };
        
        let result = eligibility_engine.evaluate_unpaid_leave_eligibility(Parameters(direct_params)).await;
        match result {
            Ok(call_result) => {
                println!("Resultado Supuesto E: {:?}", call_result);
            },
            Err(e) => panic!("Error inesperado: {}", e),
        }
    }

    #[tokio::test]
    async fn test_eligibility_engine_case_b() {
        let eligibility_engine = EligibilityEngine::new();
        let direct_params = UnpaidLeaveDirectParams {
            relationship: "mother".to_string(),
            situation: "birth".to_string(),
            is_single_parent: false,
            total_children_after: Some(3.0), // Third child
        };
        
        let result = eligibility_engine.evaluate_unpaid_leave_eligibility(Parameters(direct_params)).await;
        match result {
            Ok(call_result) => {
                println!("Resultado Supuesto B: {:?}", call_result);
            },
            Err(e) => panic!("Error inesperado: {}", e),
        }
    }

    #[tokio::test]
    async fn test_eligibility_engine_validation_error() {
        let eligibility_engine = EligibilityEngine::new();
        let direct_params = UnpaidLeaveDirectParams {
            relationship: "brother".to_string(), // Not valid
            situation: "birth".to_string(),
            is_single_parent: false,
            total_children_after: None,
        };
        
        let result = eligibility_engine.evaluate_unpaid_leave_eligibility(Parameters(direct_params)).await;
        match result {
            Ok(call_result) => {
                // Should handle validation errors appropriately
                println!("Validation result: {:?}", call_result);
            },
            Err(e) => panic!("Error inesperado: {}", e),
        }
    }
}
