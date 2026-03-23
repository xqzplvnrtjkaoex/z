use axum::{
    Json,
    extract::{Extension, State},
    response::IntoResponse,
};
use madome_proto::user::UpdateUserRequest;

use crate::{
    error::AppError, middleware::CallerContext, model::User, payload::user::UpdateUserBody,
    state::AppState,
};

pub async fn update_me(
    State(state): State<AppState>,
    Extension(caller_ctx): Extension<CallerContext>,
    Json(body): Json<UpdateUserBody>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(UpdateUserRequest {
        id: caller_ctx.caller_id.clone(),
        handle: body.handle,
        name: body.name,
    });
    caller_ctx.inject_into(&mut request);

    let response = state.user_client.clone().update_user(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
