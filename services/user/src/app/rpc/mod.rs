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

impl From<UserRole> for Role {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::User => Role::User,
            UserRole::Admin => Role::Admin,
            UserRole::Owner => Role::Owner,
        }
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id.to_string(),
            handle: user.handle,
            name: user.name,
            role: Role::from(user.role) as i32,
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
}

/// Newtype for proto role `i32` values to enable `TryFrom` conversion to domain `UserRole`.
pub(crate) struct ProtoRole(pub i32);

impl TryFrom<ProtoRole> for UserRole {
    type Error = Status;

    fn try_from(value: ProtoRole) -> Result<Self, Self::Error> {
        match Role::try_from(value.0) {
            Ok(Role::User) => Ok(UserRole::User),
            Ok(Role::Admin) => Ok(UserRole::Admin),
            Ok(Role::Owner) => Ok(UserRole::Owner),
            Ok(Role::Unspecified) | Err(_) => Err(Status::invalid_argument("invalid role value")),
        }
    }
}
