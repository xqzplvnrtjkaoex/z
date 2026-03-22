use std::sync::LazyLock;

use validator::Validate;

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

static HANDLE_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

fn check_reserved_handle(handle: &str) -> Result<(), validator::ValidationError> {
    if RESERVED_HANDLES
        .iter()
        .any(|r| r.eq_ignore_ascii_case(handle))
    {
        return Err(validator::ValidationError::new("reserved_handle"));
    }
    Ok(())
}

fn check_handle_chars(handle: &str) -> Result<(), validator::ValidationError> {
    if !HANDLE_REGEX.is_match(handle) {
        return Err(validator::ValidationError::new("invalid_handle_chars"));
    }
    Ok(())
}

fn check_name_length(name: &str) -> Result<(), validator::ValidationError> {
    let len = name.chars().count();
    if len < 1 {
        let mut err = validator::ValidationError::new("name_too_short");
        err.message = Some("name must be at least 1 character".into());
        return Err(err);
    }
    if len > 20 {
        let mut err = validator::ValidationError::new("name_too_long");
        err.message = Some("name must be at most 20 characters".into());
        return Err(err);
    }
    Ok(())
}

/// Input struct for handle validation per D-06~D-09.
/// Uses #[derive(Validate)] for declarative struct-level validation per D-12.
#[derive(Debug, Validate)]
pub struct HandleInput {
    #[validate(length(min = 4, max = 15, message = "handle must be 4-15 characters"))]
    #[validate(custom(
        function = "check_handle_chars",
        message = "handle must contain only alphanumeric characters and underscores"
    ))]
    #[validate(custom(function = "check_reserved_handle", message = "handle is reserved"))]
    pub handle: String,
}

impl HandleInput {
    pub fn new(handle: impl Into<String>) -> Self {
        Self {
            handle: handle.into(),
        }
    }

    /// Validate and return a descriptive error string on failure.
    pub fn validate_handle(&self) -> Result<(), String> {
        self.validate().map_err(|e| {
            e.field_errors()
                .values()
                .flat_map(|errs| errs.iter())
                .filter_map(|e| e.message.as_ref())
                .next()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "invalid handle".to_string())
        })
    }
}

/// Input struct for name validation per D-10~D-11.
/// Uses #[derive(Validate)] for declarative struct-level validation per D-12.
/// Note: Uses custom validator for length because D-10 requires chars().count()
/// (Unicode scalar values), not String::len() (byte length).
#[derive(Debug, Validate)]
pub struct NameInput {
    #[validate(custom(
        function = "check_name_length",
        message = "name must be 1-20 characters"
    ))]
    pub name: String,
}

impl NameInput {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Validate and return a descriptive error string on failure.
    pub fn validate_name(&self) -> Result<(), String> {
        self.validate().map_err(|e| {
            e.field_errors()
                .values()
                .flat_map(|errs| errs.iter())
                .filter_map(|e| e.message.as_ref())
                .next()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "invalid name".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // HandleInput tests
    #[test]
    fn should_reject_handle_shorter_than_4_characters() {
        let input = HandleInput::new("abc");
        assert!(input.validate_handle().is_err());
    }

    #[test]
    fn should_reject_handle_longer_than_15_characters() {
        let input = HandleInput::new("a".repeat(16));
        assert!(input.validate_handle().is_err());
    }

    #[test]
    fn should_reject_handle_with_non_alphanumeric_or_underscore_characters() {
        let input = HandleInput::new("test-user");
        assert!(input.validate_handle().is_err());

        let input2 = HandleInput::new("test user");
        assert!(input2.validate_handle().is_err());

        let input3 = HandleInput::new("test@user");
        assert!(input3.validate_handle().is_err());
    }

    #[test]
    fn should_accept_valid_handles() {
        assert!(HandleInput::new("test").validate_handle().is_ok());
        assert!(HandleInput::new("test_user1").validate_handle().is_ok());
        assert!(HandleInput::new("1234").validate_handle().is_ok());
        assert!(
            HandleInput::new("abcdefghijklmno")
                .validate_handle()
                .is_ok()
        ); // exactly 15 chars
    }

    #[test]
    fn should_reject_reserved_handles_case_insensitively() {
        assert!(HandleInput::new("me").validate_handle().is_err());
        assert!(HandleInput::new("Me").validate_handle().is_err());
        assert!(HandleInput::new("ME").validate_handle().is_err());
        assert!(HandleInput::new("admin").validate_handle().is_err());
        assert!(HandleInput::new("Admin").validate_handle().is_err());
        assert!(HandleInput::new("system").validate_handle().is_err());
        assert!(HandleInput::new("support").validate_handle().is_err());
        assert!(HandleInput::new("help").validate_handle().is_err());
        assert!(HandleInput::new("deleted").validate_handle().is_err());
        assert!(HandleInput::new("unknown").validate_handle().is_err());
        assert!(HandleInput::new("anonymous").validate_handle().is_err());
        assert!(HandleInput::new("madome").validate_handle().is_err());
    }

    // NameInput tests
    #[test]
    fn should_reject_empty_name() {
        let input = NameInput::new("");
        assert!(input.validate_name().is_err());
    }

    #[test]
    fn should_reject_name_longer_than_20_characters_by_char_count() {
        // 21 ASCII characters
        let input = NameInput::new("a".repeat(21));
        assert!(input.validate_name().is_err());

        // 21 Korean characters (each is 3 bytes in UTF-8, but 1 char)
        let input2 = NameInput::new("가".repeat(21));
        assert!(input2.validate_name().is_err());
    }

    #[test]
    fn should_accept_unicode_names_within_length() {
        // 20 Korean characters
        let input = NameInput::new("가".repeat(20));
        assert!(input.validate_name().is_ok());

        // 20 Japanese characters
        let input2 = NameInput::new("あ".repeat(20));
        assert!(input2.validate_name().is_ok());

        // Mixed
        let input3 = NameInput::new("Korean이름");
        assert!(input3.validate_name().is_ok());
    }
}
