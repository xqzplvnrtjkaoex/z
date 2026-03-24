use validator::Validate;

/// Validates a display name: 1-20 Unicode characters (counted by `chars()`, not byte length).
#[derive(Debug, Validate)]
pub struct NameInput {
    #[validate(length(min = 1, max = 20, message = "name must be 1-20 characters"))]
    pub name: String,
}

impl NameInput {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use super::*;

    #[test]
    fn should_reject_empty_name() {
        let input = NameInput::new("");
        assert!(input.validate().is_err());
    }

    #[test]
    fn should_reject_name_longer_than_20_characters_by_char_count() {
        // 21 ASCII characters
        let input = NameInput::new("a".repeat(21));
        assert!(input.validate().is_err());

        // 21 Korean characters (each is 3 bytes in UTF-8, but 1 char)
        let input2 = NameInput::new("가".repeat(21));
        assert!(input2.validate().is_err());
    }

    #[test]
    fn should_accept_unicode_names_within_length() {
        // 20 Korean characters
        let input = NameInput::new("가".repeat(20));
        assert!(input.validate().is_ok());

        // 20 Japanese characters
        let input2 = NameInput::new("あ".repeat(20));
        assert!(input2.validate().is_ok());

        // Mixed
        let input3 = NameInput::new("Korean이름");
        assert!(input3.validate().is_ok());
    }
}
