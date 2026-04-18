use std::sync::Arc;

use axum::{routing::get, Router};
use tokio::sync::Mutex;

use crate::application::products::products_service::ProductService;
use crate::config::AppConfig;
use crate::infrastructure::products::toasty_repository::ToastyProductRepository;
use crate::presentation::products::routes::product_routes;

pub struct AppState {
    pub product_service: Arc<ProductService>,
    pub jwt_secret: Arc<str>,
}

pub async fn build_app(config: &AppConfig) -> Router {
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres via Toasty. Check DATABASE_URL and Postgres status.");

    let product_repository = Arc::new(ToastyProductRepository::new(Arc::new(Mutex::new(db))));
    let product_service = Arc::new(ProductService::new(product_repository));

    let state: Arc<AppState> = Arc::new(AppState {
        product_service,
        jwt_secret: Arc::<str>::from(config.jwt_secret.clone()),
    });

    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/products", product_routes(state.clone()))
        .with_state(state)
}
