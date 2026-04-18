use std::sync::Arc;
use axum::{Router, middleware, routing::get};

use crate::app::AppState;
use crate::presentation::middleware::auth as auth_middleware;
use crate::presentation::users::handlers;

pub fn user_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/", get(handlers::list_users_handler)
                .post(handlers::create_user_handler))
        .route(
            "/{id}",
            get(handlers::get_user_by_id_handler)
                .patch(handlers::update_user_handler)
                .delete(handlers::delete_user_handler),
        )
        .route_layer(middleware::from_fn_with_state(state, auth_middleware::require_auth))
}