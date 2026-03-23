use axum::{
    Json,
    extract::{Extension, Path, State},
    response::IntoResponse,
};
use madome_core::error::AppError;
use madome_proto::user::ActivateUserRequest;

use crate::middleware::CallerContext;
use crate::model::User;
use crate::state::AppState;

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

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
