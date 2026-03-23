use axum::{
    Json,
    extract::{Extension, Path, State},
    response::IntoResponse,
};
use madome_common::caller::CallerIdentity;
use madome_proto::user::ChangeRoleRequest;

use crate::{error::AppError, model::User, payload::user::ChangeRoleBody, state::AppState};

pub async fn change_role(
    State(state): State<AppState>,
    Extension(identity): Extension<CallerIdentity>,
    Path(id): Path<String>,
    Json(body): Json<ChangeRoleBody>,
) -> Result<impl IntoResponse, AppError> {
    let new_role: i32 = body.role.into();

    let mut request = tonic::Request::new(ChangeRoleRequest { id, new_role });
    identity.inject_into(&mut request);

    let response = state.user_client.clone().change_role(request).await?;

    let user: User = response.into_inner().into();
    Ok(Json(user))
}
