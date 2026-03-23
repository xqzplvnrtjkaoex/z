use chrono::{DateTime, Utc};
use madome_proto::user::{Role, UserResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::serde::to_rfc3339_ms;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct User {
    pub id: Uuid,
    pub handle: String,
    pub name: String,
    pub role: UserRole,
    pub is_active: bool,
    #[serde(serialize_with = "to_rfc3339_ms")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "to_rfc3339_ms")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    Owner,
    Admin,
    User,
}

impl From<Role> for UserRole {
    fn from(role: Role) -> Self {
        match role {
            Role::Owner => Self::Owner,
            Role::Admin => Self::Admin,
            Role::User | Role::Unspecified => Self::User,
        }
    }
}

impl From<UserRole> for i32 {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Owner => Role::Owner as i32,
            UserRole::Admin => Role::Admin as i32,
            UserRole::User => Role::User as i32,
        }
    }
}

impl From<UserResponse> for User {
    fn from(r: UserResponse) -> Self {
        let role = Role::try_from(r.role)
            .map(UserRole::from)
            .unwrap_or(UserRole::User);
        let created_at = r
            .created_at
            .and_then(|t| DateTime::from_timestamp(t.seconds, t.nanos as u32))
            .unwrap_or_default();
        let updated_at = r
            .updated_at
            .and_then(|t| DateTime::from_timestamp(t.seconds, t.nanos as u32))
            .unwrap_or_default();
        Self {
            id: Uuid::from_slice(&r.id).unwrap_or_default(),
            handle: r.handle,
            name: r.name,
            role,
            is_active: r.is_active,
            created_at,
            updated_at,
        }
    }
}
