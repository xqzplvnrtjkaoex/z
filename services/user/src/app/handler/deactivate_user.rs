use madome_proto::user::{DeactivateUserRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::app::handler::{extract_caller_context, user_to_response};
use crate::domain::ports::UserPorts;
use crate::usecase::deactivate_user::{DeactivateUserPayload, deactivate_user};

pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<DeactivateUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let caller_ctx = extract_caller_context(&request)?;
    let req = request.into_inner();

    let target_id = req
        .id
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid user id format"))?;

    let payload = DeactivateUserPayload {
        target_id,
        caller_id: caller_ctx.caller_id,
        caller_role: caller_ctx.caller_role,
    };

    let user = deactivate_user(ctx, payload)
        .await
        .map_err(|e| e.into_status())?;

    Ok(Response::new(user_to_response(&user)))
}
