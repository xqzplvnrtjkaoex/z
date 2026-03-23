use serde::Deserialize;

use crate::model;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListUsersQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    include_inactive: Option<bool>,
}

impl ListUsersQuery {
    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(25) as i32
    }

    pub fn cursor(&mut self) -> Option<String> {
        self.cursor.take()
    }

    pub fn include_inactive(&self) -> bool {
        self.include_inactive.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateUserBody {
    pub handle: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChangeRoleBody {
    pub role: model::UserRole,
}
