use madome_common::caller::CallerIdentity;
use madome_proto::user::{ActivateUserRequest, UserResponse};
use tonic::{Request, Response, Status};

use crate::{
    domain::ports::UserPorts, payload::user::ActivateUserPayload,
    usecase::activate_user::activate_user,
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "ActivateUser"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<ActivateUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let identity = CallerIdentity::from_metadata(&request)?;
    let req = request.into_inner();
    let target_id = super::parse_user_id(&req.id)?;

    let payload =
        ActivateUserPayload::new(target_id, identity.caller_id, identity.caller_role.into());

    let user = activate_user(ctx, payload).await?;

    Ok(Response::new(user.into()))
}
