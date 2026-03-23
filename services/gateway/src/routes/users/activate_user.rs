use axum::{
    Json,
    extract::{Extension, Path, State},
    response::IntoResponse,
};
use madome_common::caller::CallerIdentity;
use madome_proto::user::ActivateUserRequest;

use crate::{error::AppError, model::User, state::AppState};

pub async fn activate_user(
    State(state): State<AppState>,
    Extension(identity): Extension<CallerIdentity>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(ActivateUserRequest { id });
    identity.inject_into(&mut request);

    let response = state.user_client.clone().activate_user(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
