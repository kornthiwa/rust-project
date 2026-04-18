use std::sync::Arc;

use axum::{Router, middleware};

use crate::app::AppState;
use crate::presentation::auth::handlers;
use crate::presentation::middleware::auth as auth_middleware;

pub fn auth_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let protected_route = axum::routing::get(handlers::me_handler).route_layer(
        middleware::from_fn_with_state(state, auth_middleware::require_auth),
    );

    Router::new()
        .route("/register", axum::routing::post(handlers::create_account_handler))
        .route("/login", axum::routing::post(handlers::login_handler))
        .route("/me", protected_route)
}
