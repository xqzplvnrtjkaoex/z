use axum::{
    Json,
    extract::{Extension, State},
    response::IntoResponse,
};
use madome_proto::user::GetUserRequest;

use crate::{error::AppError, middleware::CallerContext, model::User, state::AppState};

pub async fn get_me(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(GetUserRequest {
        id: caller_ctx.caller_id.clone(),
    });
    caller_ctx.inject_into(&mut request);

    let response = state.user_client.clone().get_user(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
