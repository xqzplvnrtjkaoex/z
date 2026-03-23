use madome_common::caller::CallerIdentity;
use madome_proto::user::{GetUserByHandleRequest, UserResponse};
use tonic::{Request, Response, Status};

use crate::{
    domain::ports::UserPorts,
    usecase::get_user_by_handle::{GetUserByHandlePayload, get_user_by_handle},
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "GetUserByHandle"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<GetUserByHandleRequest>,
) -> Result<Response<UserResponse>, Status> {
    let identity = CallerIdentity::from_metadata(&request)?;
    let req = request.into_inner();

    let payload = GetUserByHandlePayload {
        handle: req.handle,
        caller_role: identity.caller_role.into(),
    };

    let user = get_user_by_handle(ctx, payload).await?;

    Ok(Response::new(user.into()))
}
