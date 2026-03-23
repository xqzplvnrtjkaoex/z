use madome_proto::user::{GetUserRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{domain::ports::UserPorts, usecase::get_user::get_user};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "GetUser"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<GetUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let req = request.into_inner();

    let id = req
        .id
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid user id format"))?;

    let user = get_user(ctx, id).await?;

    Ok(Response::new(user.into()))
}
