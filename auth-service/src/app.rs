use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use toasty::Db;
use tower_http::catch_panic::CatchPanicLayer;

use crate::application::account::service::AccountService;
use crate::application::auth::service::AuthService;
use crate::config::config::AppConfig;
use crate::domain::account::repository::AccountRepository;
use crate::infrastructure::account::account_model::AccountModel;
use crate::infrastructure::account::postgres_repository::PostgresAccountRepository;
use crate::presentation::http::{
    account_handler, auth_handler, error::ApiError, middleware::jwt_auth_middleware,
};

#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
    pub auth_service: Arc<AuthService>,
}

pub async fn build_router(app_config: &AppConfig) -> toasty::Result<Router> {
    let state = build_app_state(app_config).await?;

    let protected_account_router = account_handler::routes().layer(middleware::from_fn_with_state(
        state.clone(),
        jwt_auth_middleware,
    ));

    let auth_branch = Router::new()
        .route("/", get(root))
        .merge(auth_handler::routes())
        .nest("/accounts", protected_account_router);

    let api_v1_router = Router::new().nest("/auth", auth_branch);

    Ok(Router::new()
        .nest("/api/v1", api_v1_router)
        .fallback(fallback_handler)
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn(print_called_path))
        .with_state(state))
}

async fn build_app_state(app_config: &AppConfig) -> toasty::Result<AppState> {
    let db = Db::builder()
        .models(toasty::models!(AccountModel))
        .connect(&app_config.database_url)
        .await?;

    if let Err(error) = db.push_schema().await {
        let error_message = error.to_string();
        if !error_message.contains("already exists") {
            return Err(error);
        }
        tracing::warn!("skip schema push: {}", error_message);
    }

    let postgres_repository = PostgresAccountRepository::new(db);
    let account_repository: Arc<dyn AccountRepository> = Arc::new(postgres_repository);
    let account_service = Arc::new(AccountService::new(Arc::clone(&account_repository)));
    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&account_repository),
        app_config.jwt_secret.clone(),
        app_config.jwt_expiration_seconds,
    ));

    Ok(AppState {
        account_service,
        auth_service,
    })
}

async fn root() -> &'static str {
    "auth-service is up"
}

async fn print_called_path(req: Request<Body>, next: Next) -> Response {
    tracing::info!("{} {}", req.method(), req.uri().path());
    next.run(req).await
}

async fn fallback_handler() -> ApiError {
    ApiError::not_found("route_not_found", "route not found")
}
