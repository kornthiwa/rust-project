mod app;
mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config:config::AppConfig = config::AppConfig::from_env();
    let app: axum::Router = app::build_app(&config).await;
    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(&config.bind_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}