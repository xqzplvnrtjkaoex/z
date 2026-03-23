use axum::{
    Json,
    extract::{Extension, Path, State},
    response::IntoResponse,
};
use madome_common::caller::CallerIdentity;
use madome_proto::user::DeactivateUserRequest;
use uuid::Uuid;

use crate::{error::AppError, model::User, state::AppState};

pub async fn deactivate_user(
    State(state): State<AppState>,
    Extension(identity): Extension<CallerIdentity>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(DeactivateUserRequest {
        id: id.as_bytes().to_vec(),
    });
    identity.inject_into(&mut request);

    let response = state.user_client.clone().deactivate_user(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
