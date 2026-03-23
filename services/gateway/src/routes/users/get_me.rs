use axum::{
    Json,
    extract::{Extension, State},
    response::IntoResponse,
};
use madome_common::caller::CallerIdentity;
use madome_proto::user::GetUserRequest;

use crate::{error::AppError, model::User, state::AppState};

pub async fn get_me(
    State(state): State<AppState>,
    Extension(identity): Extension<CallerIdentity>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(GetUserRequest {
        id: identity.caller_id.as_bytes().to_vec(),
    });
    identity.inject_into(&mut request);

    let response = state.user_client.clone().get_user(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
