use serde::Deserialize;

use crate::model;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListUsersQuery {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
    pub include_inactive: Option<bool>,
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
