use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use sqlx::postgres::PgPoolOptions;
use tower_http::catch_panic::CatchPanicLayer;

use crate::application::message::service::MessageService;
use crate::config::config::AppConfig;
use crate::domain::message::repository::MessageRepository;
use crate::infrastructure::message::postgres_repository::PostgresMessageRepository;
use crate::presentation::http::{
    error::ApiError, message_handler, middleware::jwt_auth_middleware,
};

#[derive(Clone)]
pub struct AppState {
    pub message_service: Arc<MessageService>,
    pub jwt_secret: Arc<String>,
}

pub async fn build_router(app_config: &AppConfig) -> Result<Router, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = build_app_state(app_config, pool);

    let protected_messages_router = message_handler::routes().layer(
        middleware::from_fn_with_state(state.clone(), jwt_auth_middleware),
    );

    let api_v1_router = Router::new()
        .route("/", get(root))
        .nest("/messages", protected_messages_router);

    Ok(Router::new()
        .nest("/api/v1", api_v1_router)
        .fallback(fallback_handler)
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn(print_called_path))
        .with_state(state))
}

fn build_app_state(app_config: &AppConfig, pool: sqlx::PgPool) -> AppState {
    let message_repository: Arc<dyn MessageRepository> =
        Arc::new(PostgresMessageRepository::new(pool));
    let message_service = Arc::new(MessageService::new(message_repository));
    AppState {
        message_service,
        jwt_secret: Arc::new(app_config.jwt_secret.clone()),
    }
}

async fn root() -> &'static str {
    "messages-service is up"
}

async fn print_called_path(req: Request<Body>, next: Next) -> Response {
    tracing::info!("{} {}", req.method(), req.uri().path());
    next.run(req).await
}

async fn fallback_handler() -> ApiError {
    ApiError::not_found("route_not_found", "route not found")
}
