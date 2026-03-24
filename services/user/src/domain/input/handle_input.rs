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
    if !handle.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err(validator::ValidationError::new("invalid_handle_chars"));
    }
    Ok(())
}

/// Validates a user handle: 4-15 alphanumeric/underscore chars, not reserved.
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
