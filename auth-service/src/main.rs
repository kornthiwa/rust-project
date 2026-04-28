mod app;
mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use tokio::net::TcpListener;
use crate::config::config::AppConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app_config: AppConfig = AppConfig::from_env();
    let app: axum::Router = app::build_router(&app_config).await?;

    let listener: TcpListener = TcpListener::bind(&app_config.port_config).await?;
    tracing::info!(addr = %app_config.port_config, "auth service listening");

    axum::serve(listener, app).await?;
    Ok(())
}
