pub mod activate_user;
pub mod change_role;
pub mod create_user;
pub mod deactivate_user;
pub mod get_user;
pub mod get_user_by_handle;
pub mod list_users;
pub mod update_user;

use madome_common::headers;
use madome_proto::user::{Role, UserResponse};
use prost_types::Timestamp;
use tonic::{Request, Status};
use uuid::Uuid;

use crate::domain::types::{role::UserRole, user::User};

/// Caller context extracted from gRPC metadata.
pub(crate) struct CallerContext {
    pub caller_id: Uuid,
    pub caller_role: UserRole,
}

/// Extract mandatory caller context (caller_id + caller_role) from gRPC metadata.
pub(crate) fn extract_caller_context<T>(request: &Request<T>) -> Result<CallerContext, Status> {
    let caller_id = request
        .metadata()
        .get(headers::X_CALLER_ID)
        .ok_or_else(|| Status::unauthenticated("missing x-caller-id"))?
        .to_str()
        .map_err(|_| Status::invalid_argument("invalid x-caller-id header"))?
        .parse::<Uuid>()
        .map_err(|_| Status::invalid_argument("invalid x-caller-id format"))?;

    let caller_role = request
        .metadata()
        .get(headers::X_CALLER_ROLE)
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
        .get(headers::X_CALLER_ROLE)
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
