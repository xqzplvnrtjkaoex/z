use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

use crate::domain::{error::user_error::UserError, types::role::UserRole};

// --- Handle validation ---

const RESERVED_HANDLES: &[&str] = &[
    "me",
    "admin",
    "system",
    "support",
    "help",
    "deleted",
    "unknown",
    "anonymous",
    "madome",
];

fn check_reserved_handle(handle: &str) -> Result<(), validator::ValidationError> {
    if RESERVED_HANDLES
        .iter()
        .any(|r| r.eq_ignore_ascii_case(handle))
    {
        return Err(validator::ValidationError::new("handle is reserved"));
    }
    Ok(())
}

fn check_handle_chars(handle: &str) -> Result<(), validator::ValidationError> {
    if !handle
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(validator::ValidationError::new(
            "handle must contain only alphanumeric characters and underscores",
        ));
    }
    Ok(())
}

// --- Payloads ---

#[derive(Validate)]
pub struct CreateUserPayload {
    #[validate(
        length(min = 4, max = 15, message = "handle must be 4-15 characters"),
        custom(
            function = "check_handle_chars",
            message = "handle must contain only alphanumeric characters and underscores"
        ),
        custom(function = "check_reserved_handle", message = "handle is reserved")
    )]
    handle: String,
    #[validate(length(min = 1, max = 20, message = "name must be 1-20 characters"))]
    name: String,
    role: UserRole,
}

impl CreateUserPayload {
    pub fn new(handle: String, name: String, role: UserRole) -> Self {
        Self { handle, name, role }
    }

    pub fn handle(&self) -> &str {
        &self.handle
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> UserRole {
        self.role
    }

    pub fn into_parts(self) -> (String, String, UserRole) {
        (self.handle, self.name, self.role)
    }
}

#[derive(Validate)]
pub struct UpdateUserPayload {
    id: Uuid,
    #[validate(
        length(min = 4, max = 15, message = "handle must be 4-15 characters"),
        custom(
            function = "check_handle_chars",
            message = "handle must contain only alphanumeric characters and underscores"
        ),
        custom(function = "check_reserved_handle", message = "handle is reserved")
    )]
    handle: Option<String>,
    #[validate(length(min = 1, max = 20, message = "name must be 1-20 characters"))]
    name: Option<String>,
}

impl UpdateUserPayload {
    pub fn new(id: Uuid, handle: Option<String>, name: Option<String>) -> Self {
        Self { id, handle, name }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn handle(&self) -> Option<&str> {
        self.handle.as_deref()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn into_parts(self) -> (Uuid, Option<String>, Option<String>) {
        (self.id, self.handle, self.name)
    }
}

#[derive(Validate)]
pub struct ListUsersPayload {
    #[validate(range(max = 100, message = "limit must be at most 100"))]
    limit: u64,
    cursor: Option<String>,
    include_inactive: bool,
}

impl ListUsersPayload {
    pub fn new(limit: u64, cursor: Option<String>, include_inactive: bool) -> Self {
        Self {
            limit,
            cursor,
            include_inactive,
        }
    }

    pub fn limit(&self) -> u64 {
        if self.limit == 0 { 25 } else { self.limit }
    }

    pub fn cursor(&self) -> Result<Option<(DateTime<Utc>, Uuid)>, UserError> {
        let Some(cursor_str) = &self.cursor else {
            return Ok(None);
        };
        let bytes = BASE64
            .decode(cursor_str.as_bytes())
            .map_err(|_| UserError::InvalidInput("invalid cursor encoding".to_string()))?;
        let s = String::from_utf8(bytes)
            .map_err(|_| UserError::InvalidInput("invalid cursor encoding".to_string()))?;
        let (ts_part, uuid_part) = s
            .rsplit_once(',')
            .ok_or_else(|| UserError::InvalidInput("invalid cursor format".to_string()))?;
        let ts = ts_part
            .parse::<DateTime<Utc>>()
            .map_err(|_| UserError::InvalidInput("invalid cursor timestamp".to_string()))?;
        let id = uuid_part
            .parse::<Uuid>()
            .map_err(|_| UserError::InvalidInput("invalid cursor uuid".to_string()))?;
        Ok(Some((ts, id)))
    }

    pub fn include_inactive(&self) -> bool {
        self.include_inactive
    }
}

pub struct DeactivateUserPayload {
    target_id: Uuid,
    caller_id: Uuid,
    caller_role: UserRole,
}

impl DeactivateUserPayload {
    pub fn new(target_id: Uuid, caller_id: Uuid, caller_role: UserRole) -> Self {
        Self {
            target_id,
            caller_id,
            caller_role,
        }
    }

    pub fn target_id(&self) -> Uuid {
        self.target_id
    }

    pub fn caller_id(&self) -> Uuid {
        self.caller_id
    }

    pub fn caller_role(&self) -> UserRole {
        self.caller_role
    }
}

pub struct ActivateUserPayload {
    target_id: Uuid,
    caller_id: Uuid,
    caller_role: UserRole,
}

impl ActivateUserPayload {
    pub fn new(target_id: Uuid, caller_id: Uuid, caller_role: UserRole) -> Self {
        Self {
            target_id,
            caller_id,
            caller_role,
        }
    }

    pub fn target_id(&self) -> Uuid {
        self.target_id
    }

    pub fn caller_id(&self) -> Uuid {
        self.caller_id
    }

    pub fn caller_role(&self) -> UserRole {
        self.caller_role
    }
}

pub struct ChangeRolePayload {
    target_id: Uuid,
    new_role: UserRole,
    caller_id: Uuid,
    caller_role: UserRole,
}

impl ChangeRolePayload {
    pub fn new(
        target_id: Uuid,
        new_role: UserRole,
        caller_id: Uuid,
        caller_role: UserRole,
    ) -> Self {
        Self {
            target_id,
            new_role,
            caller_id,
            caller_role,
        }
    }

    pub fn target_id(&self) -> Uuid {
        self.target_id
    }

    pub fn new_role(&self) -> UserRole {
        self.new_role
    }

    pub fn caller_id(&self) -> Uuid {
        self.caller_id
    }

    pub fn caller_role(&self) -> UserRole {
        self.caller_role
    }
}

pub struct GetUserByHandlePayload {
    handle: String,
    caller_role: UserRole,
}

impl GetUserByHandlePayload {
    pub fn new(handle: String, caller_role: UserRole) -> Self {
        Self {
            handle,
            caller_role,
        }
    }

    pub fn handle(&self) -> &str {
        &self.handle
    }

    pub fn caller_role(&self) -> UserRole {
        self.caller_role
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use validator::Validate;

    use super::*;

    // --- Handle validation ---

    #[test]
    fn should_reject_handle_shorter_than_4_characters() {
        let payload = CreateUserPayload::new("abc".to_string(), "Test".to_string(), UserRole::User);
        assert!(payload.validate().is_err());
    }

    #[test]
    fn should_reject_handle_longer_than_15_characters() {
        let payload = CreateUserPayload::new("a".repeat(16), "Test".to_string(), UserRole::User);
        assert!(payload.validate().is_err());
    }

    #[test]
    fn should_reject_handle_with_non_alphanumeric_characters() {
        for bad in ["test-user", "test user", "test@user"] {
            let payload =
                CreateUserPayload::new(bad.to_string(), "Test".to_string(), UserRole::User);
            assert!(payload.validate().is_err(), "expected rejection for {bad}");
        }
    }

    #[test]
    fn should_accept_valid_handles() {
        for good in ["test", "test_user1", "1234", "abcdefghijklmno"] {
            let payload =
                CreateUserPayload::new(good.to_string(), "Test".to_string(), UserRole::User);
            assert!(payload.validate().is_ok(), "expected acceptance for {good}");
        }
    }

    #[test]
    fn should_reject_reserved_handles_case_insensitively() {
        for reserved in ["admin", "Admin", "ADMIN", "madome", "system", "help"] {
            let payload =
                CreateUserPayload::new(reserved.to_string(), "Test".to_string(), UserRole::User);
            assert!(
                payload.validate().is_err(),
                "expected rejection for {reserved}"
            );
        }
    }

    // --- Name validation ---

    #[test]
    fn should_reject_empty_name() {
        let payload =
            CreateUserPayload::new("testuser".to_string(), "".to_string(), UserRole::User);
        assert!(payload.validate().is_err());
    }

    #[test]
    fn should_reject_name_longer_than_20_characters() {
        let payload =
            CreateUserPayload::new("testuser".to_string(), "a".repeat(21), UserRole::User);
        assert!(payload.validate().is_err());
    }

    #[test]
    fn should_accept_unicode_names_within_length() {
        for name in [
            "Korean이름",
            "가".repeat(20).as_str(),
            "あ".repeat(20).as_str(),
        ] {
            let payload =
                CreateUserPayload::new("testuser".to_string(), name.to_string(), UserRole::User);
            assert!(payload.validate().is_ok(), "expected acceptance for {name}");
        }
    }

    // --- UpdateUserPayload: Option<String> handle validation ---

    #[test]
    fn should_skip_handle_validation_when_none() {
        let payload = UpdateUserPayload::new(Uuid::new_v4(), None, None);
        payload.validate().unwrap();
    }

    #[test]
    fn should_validate_handle_when_some() {
        let payload = UpdateUserPayload::new(Uuid::new_v4(), Some("ab".to_string()), None);
        assert!(payload.validate().is_err());
    }

    // --- ListUsersPayload ---

    #[test]
    fn should_use_default_limit_25_when_zero() {
        let payload = ListUsersPayload::new(0, None, false);
        assert_eq!(payload.limit(), 25);
    }

    #[test]
    fn should_reject_limit_over_100() {
        let payload = ListUsersPayload::new(101, None, false);
        assert!(payload.validate().is_err());
    }
}
