use madome_proto::user::{ActivateUserRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    app::rpc::extract_caller_context,
    domain::ports::UserPorts,
    usecase::activate_user::{ActivateUserPayload, activate_user},
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "ActivateUser"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<ActivateUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let caller_ctx = extract_caller_context(&request)?;
    let req = request.into_inner();

    let target_id = req
        .id
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid user id format"))?;

    let payload = ActivateUserPayload {
        target_id,
        caller_id: caller_ctx.caller_id,
        caller_role: caller_ctx.caller_role,
    };

    let user = activate_user(ctx, payload).await?;

    Ok(Response::new(UserResponse::from(&user)))
}
