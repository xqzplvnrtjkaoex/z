use axum::{
    Json,
    extract::{Extension, State},
    response::IntoResponse,
};
use madome_core::error::AppError;
use madome_proto::user::GetUserRequest;

use crate::middleware::CallerContext;
use crate::model::User;
use crate::state::AppState;

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

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
