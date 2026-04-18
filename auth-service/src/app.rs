use std::sync::Arc;

use axum::{routing::get, Router};
use tokio::sync::Mutex;

use crate::application::auth::auth_service::AuthService;
use crate::config::AppConfig;
use crate::infrastructure::auth::toasty_repository::ToastyAuthRepository;
use crate::presentation::auth::routes::auth_routes;

pub struct AppState {
    pub auth_service: Arc<AuthService>,
}

pub async fn build_app(config: &AppConfig) -> Router {
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres via Toasty. Check DATABASE_URL and Postgres status.");

    let auth_repository = Arc::new(ToastyAuthRepository::new(Arc::new(Mutex::new(db))));
    let auth_service = Arc::new(AuthService::new(
        auth_repository,
        config.jwt_secret.clone(),
        config.jwt_exp_minutes,
    ));

    let state = Arc::new(AppState { auth_service });

    Router::new()
        .route("/", get(|| async { "auth-service is running" }))
        .nest("/auth", auth_routes(state.clone()))
        .with_state(state)
}
