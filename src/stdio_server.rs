use anyhow::Result;

use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

mod common;
use common::eligibility_engine::EligibilityEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Eligibility Engine MCP server using stdio transport");

    // Create an instance of our eligibility-engine router
    let service = EligibilityEngine::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}