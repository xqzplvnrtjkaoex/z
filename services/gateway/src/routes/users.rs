use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::header,
    response::IntoResponse,
};
use madome_core::error::AppError;
use madome_proto::user::{
    ActivateUserRequest, ChangeRoleRequest, DeactivateUserRequest, GetUserRequest,
    ListUsersRequest, Role, UpdateUserRequest, UserResponse,
};
use serde::{Deserialize, Serialize};

use crate::middleware::CallerContext;
use crate::state::AppState;

// --- Response types ---

#[derive(Serialize)]
pub struct UserJson {
    pub id: String,
    pub handle: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserResponse> for UserJson {
    fn from(r: UserResponse) -> Self {
        let role_str = match Role::try_from(r.role) {
            Ok(Role::User) => "user",
            Ok(Role::Admin) => "admin",
            Ok(Role::Owner) => "owner",
            _ => "unknown",
        };
        let created_at = r
            .created_at
            .and_then(|t| chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32))
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        let updated_at = r
            .updated_at
            .and_then(|t| chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32))
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        Self {
            id: r.id,
            handle: r.handle,
            name: r.name,
            role: role_str.to_string(),
            is_active: r.is_active,
            created_at,
            updated_at,
        }
    }
}

// --- Request types ---

#[derive(Deserialize)]
pub struct UpdateMeBody {
    pub handle: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i32>,
    pub cursor: Option<String>,
    #[serde(rename = "include-inactive")]
    pub include_inactive: Option<bool>,
}

#[derive(Deserialize)]
pub struct ChangeRoleBody {
    pub role: String,
}

// --- Helpers ---

fn role_str_to_proto(role: &str) -> Result<i32, AppError> {
    match role.to_lowercase().as_str() {
        "user" => Ok(Role::User as i32),
        "admin" => Ok(Role::Admin as i32),
        "owner" => Ok(Role::Owner as i32),
        _ => Err(AppError::BadRequest(format!("invalid role: {role}"))),
    }
}

// --- Self-service handlers ---

pub async fn get_me(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(GetUserRequest {
        id: caller_ctx.caller_id.clone(),
    });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .get_user(request)
        .await
        .map_err(AppError::from)?;

    let user_json: UserJson = response.into_inner().into();
    Ok(Json(user_json))
}

pub async fn update_me(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Json(body): Json<UpdateMeBody>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(UpdateUserRequest {
        id: caller_ctx.caller_id.clone(),
        handle: body.handle,
        name: body.name,
    });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .update_user(request)
        .await
        .map_err(AppError::from)?;

    let user_json: UserJson = response.into_inner().into();
    Ok(Json(user_json))
}

// --- Admin handlers ---

pub async fn list_users(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Query(query): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(25);
    let include_inactive = query.include_inactive.unwrap_or(false);

    let mut request = tonic::Request::new(ListUsersRequest {
        limit,
        cursor: query.cursor,
        include_inactive,
    });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .list_users(request)
        .await
        .map_err(AppError::from)?;

    let inner = response.into_inner();
    let users: Vec<UserJson> = inner.users.into_iter().map(UserJson::from).collect();

    let mut resp = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(cursor) = inner.next_cursor {
        resp = resp.header(
            madome_common::headers::X_NEXT_CURSOR,
            axum::http::HeaderValue::from_str(&cursor)
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("")),
        );
    }

    let body = serde_json::to_string(&users).map_err(|e| AppError::Internal(e.to_string()))?;

    resp.body(axum::body::Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn get_user(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(GetUserRequest { id });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .get_user(request)
        .await
        .map_err(AppError::from)?;

    let user_json: UserJson = response.into_inner().into();
    Ok(Json(user_json))
}

pub async fn change_role(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Path(id): Path<String>,
    Json(body): Json<ChangeRoleBody>,
) -> Result<impl IntoResponse, AppError> {
    let new_role = role_str_to_proto(&body.role)?;

    let mut request = tonic::Request::new(ChangeRoleRequest { id, new_role });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .change_role(request)
        .await
        .map_err(AppError::from)?;

    let user_json: UserJson = response.into_inner().into();
    Ok(Json(user_json))
}

pub async fn deactivate_user(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(DeactivateUserRequest { id });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .deactivate_user(request)
        .await
        .map_err(AppError::from)?;

    let user_json: UserJson = response.into_inner().into();
    Ok(Json(user_json))
}

pub async fn activate_user(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(ActivateUserRequest { id });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .activate_user(request)
        .await
        .map_err(AppError::from)?;

    let user_json: UserJson = response.into_inner().into();
    Ok(Json(user_json))
}
