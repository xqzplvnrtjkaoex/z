pub mod create;
pub mod get;
pub mod lifecycle;
pub mod list;
pub mod update;

use madome_proto::user::{
    ActivateUserRequest, ChangeRoleRequest, CreateUserRequest, DeactivateUserRequest,
    GetUserByHandleRequest, GetUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest,
    UserResponse, user_service_server::UserService,
};
use prost_types::Timestamp;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::domain::ports::UserPorts;
use crate::domain::types::role::UserRole;
use crate::domain::types::user::User;
use madome_proto::user::Role;

pub struct UserHandler<C: UserPorts> {
    pub(crate) ctx: C,
}

impl<C: UserPorts> UserHandler<C> {
    pub fn new(ctx: C) -> Self {
        Self { ctx }
    }
}

/// Caller context extracted from gRPC metadata (D-23).
pub(crate) struct CallerContext {
    pub caller_id: Uuid,
    pub caller_role: UserRole,
}

/// Extract mandatory caller context (caller_id + caller_role) from gRPC metadata.
pub(crate) fn extract_caller_context<T>(request: &Request<T>) -> Result<CallerContext, Status> {
    let caller_id = request
        .metadata()
        .get("x-caller-id")
        .ok_or_else(|| Status::unauthenticated("missing x-caller-id"))?
        .to_str()
        .map_err(|_| Status::invalid_argument("invalid x-caller-id header"))?
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid x-caller-id format"))?;

    let caller_role = request
        .metadata()
        .get("x-caller-role")
        .ok_or_else(|| Status::unauthenticated("missing x-caller-role"))?
        .to_str()
        .map_err(|_| Status::invalid_argument("invalid x-caller-role header"))?
        .parse::<UserRole>()
        .map_err(|_| Status::invalid_argument("invalid x-caller-role value"))?;

    Ok(CallerContext {
        caller_id,
        caller_role,
    })
}

/// Try to extract optional caller role from gRPC metadata.
pub(crate) fn try_extract_caller_role<T>(request: &Request<T>) -> Option<UserRole> {
    request
        .metadata()
        .get("x-caller-role")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<UserRole>().ok())
}

/// Convert domain User to proto UserResponse.
pub(crate) fn user_to_response(user: &User) -> UserResponse {
    UserResponse {
        id: user.id.to_string(),
        handle: user.handle.clone(),
        name: user.name.clone(),
        role: domain_role_to_proto(user.role) as i32,
        is_active: user.is_active,
        created_at: Some(Timestamp {
            seconds: user.created_at.timestamp(),
            nanos: user.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(Timestamp {
            seconds: user.updated_at.timestamp(),
            nanos: user.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn domain_role_to_proto(role: UserRole) -> Role {
    match role {
        UserRole::User => Role::User,
        UserRole::Admin => Role::Admin,
        UserRole::Owner => Role::Owner,
    }
}

/// Convert proto Role int to domain UserRole.
pub(crate) fn proto_role_to_domain(role: i32) -> Result<UserRole, Status> {
    match Role::try_from(role) {
        Ok(Role::User) => Ok(UserRole::User),
        Ok(Role::Admin) => Ok(UserRole::Admin),
        Ok(Role::Owner) => Ok(UserRole::Owner),
        Ok(Role::Unspecified) | Err(_) => Err(Status::invalid_argument("invalid role value")),
    }
}

#[tonic::async_trait]
impl<C: UserPorts + Send + Sync + 'static> UserService for UserHandler<C> {
    async fn health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        create::handle(&self.ctx, request).await
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        get::handle_get(&self.ctx, request).await
    }

    async fn get_user_by_handle(
        &self,
        request: Request<GetUserByHandleRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        get::handle_get_by_handle(&self.ctx, request).await
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        list::handle(&self.ctx, request).await
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        update::handle(&self.ctx, request).await
    }

    async fn deactivate_user(
        &self,
        request: Request<DeactivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        lifecycle::handle_deactivate(&self.ctx, request).await
    }

    async fn activate_user(
        &self,
        request: Request<ActivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        lifecycle::handle_activate(&self.ctx, request).await
    }

    async fn change_role(
        &self,
        request: Request<ChangeRoleRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        lifecycle::handle_change_role(&self.ctx, request).await
    }
}
