use crate::state::AppState;
use axum::{Router, routing::get};

pub mod health;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::gateway_health))
        .nest("/v1", v1_routes())
        .with_state(state)
}

fn v1_routes() -> Router<AppState> {
    Router::new().route("/health/services", get(health::service_health))
}
