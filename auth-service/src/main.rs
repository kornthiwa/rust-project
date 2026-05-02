mod app;
mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use crate::config::config::AppConfig;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let app_config: AppConfig = AppConfig::from_env()?;
    let app: axum::Router = app::build_router(&app_config).await?;

    let listener: TcpListener = TcpListener::bind(app_config.port_config()).await?;
    tracing::info!(addr = %app_config.port_config(), "auth service listening");

    axum::serve(listener, app).await?;
    Ok(())
}
