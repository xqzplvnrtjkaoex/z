use madome_proto::user::{CreateUserRequest, UserResponse};
use tonic::{Request, Response, Status};

use crate::app::handler::{proto_role_to_domain, user_to_response};
use crate::domain::ports::UserPorts;
use crate::usecase::create_user::{CreateUserPayload, create_user};

pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<CreateUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let req = request.into_inner();

    let role = proto_role_to_domain(req.role)?;

    let payload = CreateUserPayload {
        handle: req.handle,
        name: req.name,
        role,
    };

    let user = create_user(ctx, payload)
        .await
        .map_err(|e| e.into_status())?;

    Ok(Response::new(user_to_response(&user)))
}
