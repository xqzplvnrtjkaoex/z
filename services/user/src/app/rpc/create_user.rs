use madome_proto::user::{CreateUserRequest, UserResponse};
use tonic::{Request, Response, Status};

use crate::{
    app::rpc::ProtoRole,
    domain::{ports::UserPorts, types::role::UserRole},
    usecase::create_user::{CreateUserPayload, create_user},
};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "CreateUser"))]
pub async fn execute<C: UserPorts>(
    ctx: &C,
    request: Request<CreateUserRequest>,
) -> Result<Response<UserResponse>, Status> {
    let req = request.into_inner();

    let role = UserRole::try_from(ProtoRole(req.role))?;

    let payload = CreateUserPayload {
        handle: req.handle,
        name: req.name,
        role,
    };

    let user = create_user(ctx, payload).await?;

    Ok(Response::new(UserResponse::from(&user)))
}
