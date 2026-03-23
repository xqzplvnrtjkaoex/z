use madome_proto::user::{UpdateUserRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    domain::ports::UserPorts,
    usecase::update_user::{UpdateUserPayload, update_user},
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "UpdateUser"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<UpdateUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let req = request.into_inner();

    let id = req
        .id
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid user id format"))?;

    let payload = UpdateUserPayload {
        id,
        handle: req.handle,
        name: req.name,
    };

    let user = update_user(ctx, payload).await?;

    Ok(Response::new(UserResponse::from(&user)))
}
