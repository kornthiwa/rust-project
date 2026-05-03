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

use crate::application::user::service::UserService;
use crate::config::config::AppConfig;
use crate::domain::user::repository::UserRepository;
use crate::infrastructure::user::postgres_repository::PostgresUserRepository;
use crate::presentation::http::{error::ApiError, middleware::jwt_auth_middleware, user_handler};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub jwt_secret: Arc<String>,
}

pub async fn build_router(app_config: &AppConfig) -> Result<Router, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = build_app_state(app_config, pool);

    let protected_user_router = user_handler::routes().layer(middleware::from_fn_with_state(
        state.clone(),
        jwt_auth_middleware,
    ));

    let api_v1_router = Router::new()
        .route("/", get(root))
        .nest("/user", protected_user_router);

    Ok(Router::new()
        .nest("/api/v1", api_v1_router)
        .fallback(fallback_handler)
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn(print_called_path))
        .with_state(state))
}

fn build_app_state(app_config: &AppConfig, pool: sqlx::PgPool) -> AppState {
    let user_repository: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool));
    let user_service = Arc::new(UserService::new(user_repository));
    AppState {
        user_service,
        jwt_secret: Arc::new(app_config.jwt_secret.clone()),
    }
}

async fn root() -> &'static str {
    "users-service is up"
}

async fn print_called_path(req: Request<Body>, next: Next) -> Response {
    tracing::info!("{} {}", req.method(), req.uri().path());
    next.run(req).await
}

async fn fallback_handler() -> ApiError {
    ApiError::not_found("route_not_found", "route not found")
}
