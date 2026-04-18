use std::sync::Arc;
use axum::{routing::get, Router};
use tokio::sync::Mutex;

use crate::application::users::users_service::UserService;
use crate::config::AppConfig;
use crate::infrastructure::users::toasty_repository::ToastyUserRepository;
use crate::presentation::users::routes::user_routes;

pub struct AppState {
    pub user_service: Arc<UserService>,
    pub jwt_secret: Arc<str>,
}

pub async fn build_app(config: &AppConfig) -> Router {
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres via Toasty. Check DATABASE_URL and Postgres status.");

    let user_repository = Arc::new(ToastyUserRepository::new(Arc::new(Mutex::new(db))));
    let user_service = Arc::new(UserService::new(user_repository));

    let state: Arc<AppState> = Arc::new(AppState {
        user_service,
        jwt_secret: Arc::<str>::from(config.jwt_secret.clone()),
    });

    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/users", user_routes(state.clone()))
        .with_state(state)
}