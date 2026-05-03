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

use crate::application::ports::{UserEventInboundHandlerRef, UserEventPublisher};
use crate::application::user::service::UserService;
use crate::config::config::AppConfig;
use crate::domain::user::repository::UserRepository;
use crate::infrastructure::messaging::{
    KafkaUserEventPublisher, LoggingUserEventInboundHandler, NoopUserEventPublisher,
};
use crate::infrastructure::user::postgres_repository::PostgresUserRepository;
use crate::presentation::http::{error::ApiError, middleware::jwt_auth_middleware, user_handler};

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
    pub user_service: Arc<UserService>,
    pub jwt_secret: Arc<String>,
}

pub struct Bootstrap {
    pub router: Router,
    pub user_event_inbound_handler: UserEventInboundHandlerRef,
}

pub async fn bootstrap(app_config: &AppConfig) -> Result<Bootstrap, BuildError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let user_event_inbound_handler: UserEventInboundHandlerRef =
        Arc::new(LoggingUserEventInboundHandler);
    let state = build_app_state(app_config, pool)?;

    let protected_user_router = user_handler::routes().layer(middleware::from_fn_with_state(
        state.clone(),
        jwt_auth_middleware,
    ));

    let api_v1_router = Router::new()
        .route("/", get(root))
        .nest("/user", protected_user_router);

    let router = Router::new()
        .nest("/api/v1", api_v1_router)
        .fallback(fallback_handler)
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn(print_called_path))
        .with_state(state);

    Ok(Bootstrap {
        router,
        user_event_inbound_handler,
    })
}

/// HTTP stack only; prefer [`bootstrap`] when wiring the Kafka consumer.
#[allow(dead_code)]
pub async fn build_router(app_config: &AppConfig) -> Result<Router, BuildError> {
    Ok(bootstrap(app_config).await?.router)
}

fn build_app_state(app_config: &AppConfig, pool: sqlx::PgPool) -> Result<AppState, BuildError> {
    let user_events: Arc<dyn UserEventPublisher> = if app_config.kafka_enabled {
        Arc::new(KafkaUserEventPublisher::try_new(app_config)?)
    } else {
        Arc::new(NoopUserEventPublisher)
    };

    let user_repository: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool));
    let user_service = Arc::new(UserService::new(
        user_repository,
        Arc::clone(&user_events),
    ));
    Ok(AppState {
        user_service,
        jwt_secret: Arc::new(app_config.jwt_secret.clone()),
    })
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
