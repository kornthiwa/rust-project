use std::fmt;
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

use crate::application::account::service::AccountService;
use crate::application::auth::service::AuthService;
use crate::application::ports::{
    AuthEventInboundHandlerRef, AuthEventPublisher,
};
use crate::config::config::AppConfig;
use crate::domain::account::repository::AccountRepository;
use crate::infrastructure::account::postgres_repository::PostgresAccountRepository;
use crate::infrastructure::messaging::{
    KafkaAuthEventPublisher, LoggingAuthEventInboundHandler, NoopAuthEventPublisher,
};
use crate::presentation::http::{
    account_handler, auth_handler, error::ApiError, middleware::jwt_auth_middleware,
};

#[derive(Debug)]
pub enum BuildError {
    Sqlx(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
    Kafka(rdkafka::error::KafkaError),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::Sqlx(e) => write!(f, "database: {e}"),
            BuildError::Migrate(e) => write!(f, "migration: {e}"),
            BuildError::Kafka(e) => write!(f, "kafka: {e}"),
        }
    }
}

impl std::error::Error for BuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BuildError::Sqlx(e) => Some(e),
            BuildError::Migrate(e) => Some(e),
            BuildError::Kafka(e) => Some(e),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for BuildError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        BuildError::Migrate(value)
    }
}

impl From<sqlx::Error> for BuildError {
    fn from(value: sqlx::Error) -> Self {
        BuildError::Sqlx(value)
    }
}

impl From<rdkafka::error::KafkaError> for BuildError {
    fn from(value: rdkafka::error::KafkaError) -> Self {
        BuildError::Kafka(value)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
    pub auth_service: Arc<AuthService>,
}

pub struct Bootstrap {
    pub router: Router,
    pub auth_event_inbound_handler: AuthEventInboundHandlerRef,
}

pub async fn bootstrap(app_config: &AppConfig) -> Result<Bootstrap, BuildError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let auth_event_inbound_handler: AuthEventInboundHandlerRef =
        Arc::new(LoggingAuthEventInboundHandler);
    let state = build_app_state(app_config, pool)?;

    let protected_account_router = account_handler::routes().layer(middleware::from_fn_with_state(
        state.clone(),
        jwt_auth_middleware,
    ));

    let auth_branch = Router::new()
        .route("/", get(root))
        .merge(auth_handler::routes())
        .nest("/accounts", protected_account_router);

    let api_v1_router = Router::new().nest("/auth", auth_branch);

    let router = Router::new()
        .nest("/api/v1", api_v1_router)
        .fallback(fallback_handler)
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn(print_called_path))
        .with_state(state);

    Ok(Bootstrap {
        router,
        auth_event_inbound_handler,
    })
}

/// HTTP stack only; prefer [`bootstrap`] when wiring the Kafka consumer.
#[allow(dead_code)]
pub async fn build_router(app_config: &AppConfig) -> Result<Router, BuildError> {
    Ok(bootstrap(app_config).await?.router)
}

fn build_app_state(app_config: &AppConfig, pool: sqlx::PgPool) -> Result<AppState, BuildError> {
    let auth_event_publisher: Arc<dyn AuthEventPublisher> = if app_config.kafka_enabled {
        Arc::new(KafkaAuthEventPublisher::try_new(app_config)?)
    } else {
        Arc::new(NoopAuthEventPublisher)
    };

    let postgres_repository = PostgresAccountRepository::new(pool);
    let account_repository: Arc<dyn AccountRepository> = Arc::new(postgres_repository);
    let account_service = Arc::new(AccountService::new(Arc::clone(&account_repository)));
    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&account_repository),
        app_config.jwt_secret.clone(),
        app_config.jwt_expiration_seconds,
        Arc::clone(&auth_event_publisher),
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
