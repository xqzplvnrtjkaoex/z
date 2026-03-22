use madome_proto::user::{ChangeRoleRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::app::handler::{extract_caller_context, proto_role_to_domain, user_to_response};
use crate::domain::ports::UserPorts;
use crate::usecase::change_role::{ChangeRolePayload, change_role};

pub async fn handle<C: UserPorts>(
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

    let user = change_role(ctx, payload)
        .await
        .map_err(|e| e.into_status())?;

    Ok(Response::new(user_to_response(&user)))
}
