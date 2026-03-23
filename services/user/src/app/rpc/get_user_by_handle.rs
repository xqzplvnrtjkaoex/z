use madome_proto::user::{GetUserByHandleRequest, UserResponse};
use tonic::{Request, Response, Status};

use crate::{
    app::rpc::{try_extract_caller_role, user_to_response},
    domain::ports::UserPorts,
    usecase::get_user_by_handle::{GetUserByHandlePayload, get_user_by_handle},
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "GetUserByHandle"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<GetUserByHandleRequest>,
) -> Result<Response<UserResponse>, Status> {
    let caller_role = try_extract_caller_role(&request);
    let req = request.into_inner();

    let payload = GetUserByHandlePayload {
        handle: req.handle,
        caller_role,
    };

    let user = get_user_by_handle(ctx, payload).await?;

    Ok(Response::new(user_to_response(&user)))
}
