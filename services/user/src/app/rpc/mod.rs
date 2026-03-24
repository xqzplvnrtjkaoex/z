pub mod activate_user;
pub mod change_role;
pub mod create_user;
pub mod deactivate_user;
pub mod get_user;
pub mod get_user_by_handle;
pub mod list_users;
pub mod update_user;

use madome_proto::user::{Role, UserResponse};
use prost_types::Timestamp;
use tonic::Status;
use uuid::Uuid;

use crate::domain::types::{role::UserRole, user::User};

pub(crate) fn parse_user_id(bytes: &[u8]) -> Result<Uuid, Status> {
    Uuid::from_slice(bytes).map_err(|_| Status::invalid_argument("invalid user id format"))
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
            id: user.id.as_bytes().to_vec(),
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
