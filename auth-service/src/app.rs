use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use toasty::Db;

use crate::application::account::service::AccountService;
use crate::config::config::AppConfig;
use crate::domain::account::repository::AccountRepository;
use crate::domain::account::entity::Account;
use crate::infrastructure::account::postgres_repository::PostgresAccountRepository;
use crate::presentation::http::account_handler;

#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
}

pub async fn build_router(app_config: &AppConfig) -> toasty::Result<Router> {
    let state = build_app_state(app_config).await?;
    let auth_router = Router::new()
        .route("/", get(root))
        .nest("/accounts", account_handler::routes());

    Ok(Router::new()
        .nest("/auth", auth_router)
        .with_state(state))
}

async fn build_app_state(app_config: &AppConfig) -> toasty::Result<AppState> {
    let db: Db = Db::builder()
        .models(toasty::models!(Account))
        .connect(&app_config.database_url)
        .await?;
    db.push_schema().await?;

    let postgres_repository = PostgresAccountRepository::new(db);
    let account_repository: Arc<dyn AccountRepository> = Arc::new(postgres_repository);
    let account_service = Arc::new(AccountService::new(account_repository));

    Ok(AppState { account_service })
}

async fn root() -> &'static str {
    "hello"
}