use madome_common::caller::CallerIdentity;
use madome_proto::user::{ChangeRoleRequest, UserResponse};
use tonic::{Request, Response, Status};

use crate::{
    app::rpc::ProtoRole,
    domain::{ports::UserPorts, types::role::UserRole},
    payload::user::ChangeRolePayload,
    usecase::change_role::change_role,
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "ChangeRole"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<ChangeRoleRequest>,
) -> Result<Response<UserResponse>, Status> {
    let identity = CallerIdentity::from_metadata(&request)?;
    let req = request.into_inner();
    let target_id = super::parse_user_id(&req.id)?;

    let new_role = UserRole::try_from(ProtoRole(req.new_role))?;

    let payload = ChangeRolePayload::new(
        target_id,
        new_role,
        identity.caller_id,
        identity.caller_role.into(),
    );

    let user = change_role(ctx, payload).await?;

    Ok(Response::new(user.into()))
}
