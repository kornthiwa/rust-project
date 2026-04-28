mod app;
mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use tokio::net::TcpListener;
use crate::config::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config: AppConfig = AppConfig::from_env();
    let app: axum::Router = app::build_router(&app_config).await?;

    let listener: TcpListener = TcpListener::bind(&app_config.port_config).await?;
    println!("Auth service running on http://{}", &app_config.port_config);

    axum::serve(listener, app).await?;
    Ok(())
}
