use axum::{
    Json,
    extract::{Extension, Path, State},
    response::IntoResponse,
};
use madome_proto::user::DeactivateUserRequest;

use crate::{error::AppError, middleware::CallerContext, model::User, state::AppState};

pub async fn deactivate_user(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(DeactivateUserRequest { id });
    caller_ctx.inject_into(&mut request);

    let response = state.user_client.clone().deactivate_user(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
