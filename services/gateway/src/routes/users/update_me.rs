use axum::{
    Json,
    extract::{Extension, State},
    response::IntoResponse,
};
use madome_common::caller::CallerIdentity;
use madome_proto::user::UpdateUserRequest;

use crate::{error::AppError, model::User, payload::user::UpdateUserBody, state::AppState};

pub async fn update_me(
    State(state): State<AppState>,
    Extension(identity): Extension<CallerIdentity>,
    Json(body): Json<UpdateUserBody>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(UpdateUserRequest {
        id: identity.caller_id.to_string(),
        handle: body.handle,
        name: body.name,
    });
    identity.inject_into(&mut request);

    let response = state.user_client.clone().update_user(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
