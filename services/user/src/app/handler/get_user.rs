use madome_proto::user::{GetUserByHandleRequest, GetUserRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::app::handler::{try_extract_caller_role, user_to_response};
use crate::domain::ports::UserPorts;
use crate::usecase::get_user::get_user;
use crate::usecase::get_user_by_handle::{GetUserByHandlePayload, get_user_by_handle};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "GetUser"))]
pub async fn handle_get<C: UserPorts>(
    ctx: &C,
    request: Request<GetUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let req = request.into_inner();

    let id = req
        .id
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid user id format"))?;

    let user = get_user(ctx, id).await.map_err(|e| e.into_status())?;

    Ok(Response::new(user_to_response(&user)))
}

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "GetUserByHandle"))]
pub async fn handle_get_by_handle<C: UserPorts>(
    ctx: &C,
    request: Request<GetUserByHandleRequest>,
) -> Result<Response<UserResponse>, Status> {
    let caller_role = try_extract_caller_role(&request);
    let req = request.into_inner();

    let payload = GetUserByHandlePayload {
        handle: req.handle,
        caller_role,
    };

    let user = get_user_by_handle(ctx, payload)
        .await
        .map_err(|e| e.into_status())?;

    Ok(Response::new(user_to_response(&user)))
}
