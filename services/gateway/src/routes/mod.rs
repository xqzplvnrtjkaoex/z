use axum::{
    Router, middleware,
    routing::{get, patch, post},
};

use crate::{middleware::caller_identity, state::AppState};

pub mod health;
pub mod users;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::gateway_health))
        .nest("/v1", v1_routes())
        .with_state(state)
}

fn v1_routes() -> Router<AppState> {
    Router::new()
        .route("/health/services", get(health::service_health))
        .nest("/users", user_routes())
}

fn user_routes() -> Router<AppState> {
    // Self-service routes — will get authenticated tier middleware in Phase 3
    let me_routes = Router::new().route("/@me", get(users::get_me).patch(users::update_me));

    // Admin routes — will get admin+ tier middleware in Phase 3
    let admin_routes = Router::new()
        .route("/", get(users::list_users))
        .route("/{id}", get(users::get_user))
        .route("/{id}/role", patch(users::change_role))
        .route("/{id}/deactivate", post(users::deactivate_user))
        .route("/{id}/activate", post(users::activate_user));

    Router::new()
        .merge(me_routes)
        .merge(admin_routes)
        .layer(middleware::from_fn(
            caller_identity::extract_caller_identity,
        ))
}
