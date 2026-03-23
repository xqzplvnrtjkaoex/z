use validator::Validate;

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

/// Validates a display name: 1-20 Unicode characters (counted by `chars()`, not byte length).
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
