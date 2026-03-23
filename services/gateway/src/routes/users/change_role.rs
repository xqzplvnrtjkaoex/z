use axum::{
    Json,
    extract::{Extension, Path, State},
    response::IntoResponse,
};
use madome_core::error::AppError;
use madome_proto::user::ChangeRoleRequest;

use crate::middleware::CallerContext;
use crate::model::User;
use crate::payload::user::ChangeRoleBody;
use crate::state::AppState;

pub async fn change_role(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Path(id): Path<String>,
    Json(body): Json<ChangeRoleBody>,
) -> Result<impl IntoResponse, AppError> {
    let new_role: i32 = body.role.into();

    let mut request = tonic::Request::new(ChangeRoleRequest { id, new_role });
    caller_ctx.inject_into(&mut request);

    let response = state
        .user_client
        .clone()
        .change_role(request)
        .await
        .map_err(AppError::from)?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
