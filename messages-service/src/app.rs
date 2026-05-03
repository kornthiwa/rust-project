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

use crate::application::message::service::MessageService;
use crate::application::ports::MessageEventPublisher;
use crate::config::config::AppConfig;
use crate::domain::message::repository::MessageRepository;
use crate::infrastructure::message::postgres_repository::PostgresMessageRepository;
use crate::infrastructure::messaging::{KafkaMessageEventPublisher, NoopMessageEventPublisher};
use crate::presentation::http::{
    error::ApiError, message_handler, middleware::jwt_auth_middleware,
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
    pub message_service: Arc<MessageService>,
    pub jwt_secret: Arc<String>,
}

pub async fn build_router(app_config: &AppConfig) -> Result<Router, BuildError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = build_app_state(app_config, pool)?;

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

fn build_app_state(app_config: &AppConfig, pool: sqlx::PgPool) -> Result<AppState, BuildError> {
    let message_events: Arc<dyn MessageEventPublisher> = if app_config.kafka_enabled {
        Arc::new(KafkaMessageEventPublisher::try_new(app_config)?)
    } else {
        Arc::new(NoopMessageEventPublisher)
    };

    let message_repository: Arc<dyn MessageRepository> =
        Arc::new(PostgresMessageRepository::new(pool));
    let message_service = Arc::new(MessageService::new(
        message_repository,
        Arc::clone(&message_events),
    ));
    Ok(AppState {
        message_service,
        jwt_secret: Arc::new(app_config.jwt_secret.clone()),
    })
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
