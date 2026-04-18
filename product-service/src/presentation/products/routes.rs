use std::sync::Arc;

use axum::{Router, middleware, routing::get};

use crate::app::AppState;
use crate::presentation::middleware::auth as auth_middleware;
use crate::presentation::products::handlers;

pub fn product_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            get(handlers::list_products_handler).post(handlers::create_product_handler),
        )
        .route(
            "/{id}",
            get(handlers::get_product_by_id_handler)
                .patch(handlers::update_product_handler)
                .delete(handlers::delete_product_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            auth_middleware::require_auth,
        ))
}
