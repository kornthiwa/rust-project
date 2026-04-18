mod app;
mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = config::AppConfig::from_env();
    let app = app::build_app(&config).await;
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("server failed");
}
