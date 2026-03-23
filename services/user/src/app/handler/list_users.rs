use madome_proto::user::{ListUsersRequest, ListUsersResponse, UserResponse};
use tonic::{Request, Response, Status};

use crate::app::handler::user_to_response;
use crate::domain::ports::UserPorts;
use crate::usecase::list_users::{ListUsersPayload, list_users};

#[tracing::instrument(skip_all, fields(otel.kind = "server", rpc = "ListUsers"))]
pub async fn handle<C: UserPorts>(
    ctx: &C,
    request: Request<ListUsersRequest>,
) -> Result<Response<ListUsersResponse>, Status> {
    let req = request.into_inner();

    let payload = ListUsersPayload {
        limit: req.limit as u64,
        cursor: req.cursor,
        include_inactive: req.include_inactive,
    };

    let (users, next_cursor) = list_users(ctx, payload).await?;

    let user_responses: Vec<UserResponse> = users.iter().map(user_to_response).collect();

    Ok(Response::new(ListUsersResponse {
        users: user_responses,
        next_cursor,
    }))
}
