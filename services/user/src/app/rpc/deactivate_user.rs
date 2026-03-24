use madome_common::caller::CallerIdentity;
use madome_proto::user::{DeactivateUserRequest, UserResponse};
use tonic::{Request, Response, Status};

use crate::{
    domain::ports::UserPorts, payload::user::DeactivateUserPayload,
    usecase::deactivate_user::deactivate_user,
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "DeactivateUser"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<DeactivateUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let identity = CallerIdentity::from_metadata(&request)?;
    let req = request.into_inner();
    let target_id = super::parse_user_id(&req.id)?;

    let payload = DeactivateUserPayload {
        target_id,
        caller_id: identity.caller_id,
        caller_role: identity.caller_role.into(),
    };

    let user = deactivate_user(ctx, payload).await?;

    Ok(Response::new(user.into()))
}
