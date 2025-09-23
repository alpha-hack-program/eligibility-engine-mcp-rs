use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};
mod common;
use common::{eligibility_engine::EligibilityEngine, metrics};
use axum::{response::IntoResponse, http::StatusCode};

const BIND_ADDRESS: &str = "127.0.0.1:8001";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Use environment variable or the static value
    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| BIND_ADDRESS.to_string());
    tracing::info!("Starting streamable-http Eligibility Engine MCP server on {}", bind_address);
    let service = StreamableHttpService::new(
        || Ok(EligibilityEngine::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new()
        .nest_service("/mcp", service)
        .route("/metrics", axum::routing::get(metrics_handler));
    let tcp_listener = tokio::net::TcpListener::bind(bind_address).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await;
    Ok(())
}

/// Handler for the /metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    let output = metrics::METRICS.gather();
    (StatusCode::OK, output)
}