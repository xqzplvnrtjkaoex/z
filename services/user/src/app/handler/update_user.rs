use madome_proto::user::{UpdateUserRequest, UserResponse};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::app::handler::user_to_response;
use crate::domain::ports::UserPorts;
use crate::usecase::update_user::{UpdateUserPayload, update_user};

pub async fn handle<C: UserPorts>(
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

    let user = update_user(ctx, payload)
        .await
        .map_err(|e| e.into_status())?;

    Ok(Response::new(user_to_response(&user)))
}
