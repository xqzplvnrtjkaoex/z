use axum::{Json, extract::State};
use madome_common::headers;
use serde::Serialize;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Serialize)]
pub struct GatewayHealthResponse {
    pub status: &'static str,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub request_id: String,
    pub services: ServiceStatuses,
}

#[derive(Serialize)]
pub struct ServiceStatuses {
    pub auth: &'static str,
    pub catalog: &'static str,
    pub user: &'static str,
}

pub async fn gateway_health() -> Json<GatewayHealthResponse> {
    Json(GatewayHealthResponse { status: "ok" })
}

pub async fn service_health(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, AppError> {
    let request_id = Uuid::now_v7().to_string();

    // Call each backend service health RPC, treating individual failures as "unavailable"
    let auth_status = call_auth_health(state.auth_client.clone(), &request_id).await;
    let catalog_status = call_catalog_health(state.catalog_client.clone(), &request_id).await;
    let user_status = call_user_health(state.user_client.clone(), &request_id).await;

    Ok(Json(HealthResponse {
        status: "ok",
        request_id,
        services: ServiceStatuses {
            auth: auth_status,
            catalog: catalog_status,
            user: user_status,
        },
    }))
}

async fn call_auth_health(
    mut client: madome_proto::auth::auth_service_client::AuthServiceClient<
        tonic::transport::Channel,
    >,
    request_id: &str,
) -> &'static str {
    let mut request = tonic::Request::new(());
    request
        .metadata_mut()
        .insert(headers::X_REQUEST_ID, request_id.parse().unwrap());
    match client.health(request).await {
        Ok(_) => "ok",
        Err(_) => "unavailable",
    }
}

async fn call_catalog_health(
    mut client: madome_proto::catalog::catalog_service_client::CatalogServiceClient<
        tonic::transport::Channel,
    >,
    request_id: &str,
) -> &'static str {
    let mut request = tonic::Request::new(());
    request
        .metadata_mut()
        .insert(headers::X_REQUEST_ID, request_id.parse().unwrap());
    match client.health(request).await {
        Ok(_) => "ok",
        Err(_) => "unavailable",
    }
}

async fn call_user_health(
    mut client: madome_proto::user::user_service_client::UserServiceClient<
        tonic::transport::Channel,
    >,
    request_id: &str,
) -> &'static str {
    let mut request = tonic::Request::new(());
    request
        .metadata_mut()
        .insert(headers::X_REQUEST_ID, request_id.parse().unwrap());
    match client.health(request).await {
        Ok(_) => "ok",
        Err(_) => "unavailable",
    }
}
