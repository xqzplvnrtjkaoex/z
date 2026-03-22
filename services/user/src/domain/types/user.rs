use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::role::UserRole;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub handle: String,
    pub name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
