use madome_proto::user::{ChangeRoleRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    app::rpc::{extract_caller_context, proto_role_to_domain, user_to_response},
    domain::ports::UserPorts,
    usecase::change_role::{ChangeRolePayload, change_role},
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "ChangeRole"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<ChangeRoleRequest>,
) -> Result<Response<UserResponse>, Status> {
    let caller_ctx = extract_caller_context(&request)?;
    let req = request.into_inner();

    let target_id = req
        .id
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid user id format"))?;

    let new_role = proto_role_to_domain(req.new_role)?;

    let payload = ChangeRolePayload {
        target_id,
        new_role,
        caller_id: caller_ctx.caller_id,
        caller_role: caller_ctx.caller_role,
    };

    let user = change_role(ctx, payload).await?;

    Ok(Response::new(user_to_response(&user)))
}
