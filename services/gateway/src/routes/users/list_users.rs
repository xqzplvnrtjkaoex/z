use axum::{
    extract::{Extension, State},
    http::header,
    response::IntoResponse,
};
use madome_proto::user::ListUsersRequest;
use serde_qs::axum::QsQuery;

use crate::{
    error::AppError, middleware::CallerContext, model::User, payload::user::ListUsersQuery,
    state::AppState,
};

pub async fn list_users(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    QsQuery(query): QsQuery<ListUsersQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(25) as i32;
    let include_inactive = query.include_inactive.unwrap_or(false);

    let mut request = tonic::Request::new(ListUsersRequest {
        limit,
        cursor: query.cursor,
        include_inactive,
    });
    caller_ctx.inject_into(&mut request);

    let response = state.user_client.clone().list_users(request).await?;

    let inner = response.into_inner();
    let users: Vec<User> = inner.users.into_iter().map(User::from).collect();

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
