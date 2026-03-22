use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    User,
    Admin,
    Owner,
}

impl UserRole {
    /// Returns the hierarchy level (higher = more privilege)
    pub fn level(&self) -> u8 {
        match self {
            UserRole::User => 0,
            UserRole::Admin => 1,
            UserRole::Owner => 2,
        }
    }

    /// Returns true if self can manage (modify roles/deactivate) a user with target_role.
    /// Requires strictly greater privilege level.
    pub fn can_manage(&self, target_role: UserRole) -> bool {
        self.level() > target_role.level()
    }
}

impl PartialOrd for UserRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Owner => write!(f, "owner"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(UserRole::User),
            "admin" => Ok(UserRole::Admin),
            "owner" => Ok(UserRole::Owner),
            _ => Err(format!("invalid role: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_order_owner_greater_than_admin() {
        assert!(UserRole::Owner > UserRole::Admin);
    }

    #[test]
    fn should_order_admin_greater_than_user() {
        assert!(UserRole::Admin > UserRole::User);
    }

    #[test]
    fn should_order_owner_greater_than_user() {
        assert!(UserRole::Owner > UserRole::User);
    }

    #[test]
    fn should_return_correct_levels() {
        assert_eq!(UserRole::User.level(), 0);
        assert_eq!(UserRole::Admin.level(), 1);
        assert_eq!(UserRole::Owner.level(), 2);
    }

    #[test]
    fn should_can_manage_return_true_when_caller_role_is_greater() {
        assert!(UserRole::Owner.can_manage(UserRole::Admin));
        assert!(UserRole::Owner.can_manage(UserRole::User));
        assert!(UserRole::Admin.can_manage(UserRole::User));
    }

    #[test]
    fn should_can_manage_return_false_when_caller_role_is_equal() {
        assert!(!UserRole::Owner.can_manage(UserRole::Owner));
        assert!(!UserRole::Admin.can_manage(UserRole::Admin));
        assert!(!UserRole::User.can_manage(UserRole::User));
    }

    #[test]
    fn should_can_manage_return_false_when_caller_role_is_lesser() {
        assert!(!UserRole::Admin.can_manage(UserRole::Owner));
        assert!(!UserRole::User.can_manage(UserRole::Admin));
        assert!(!UserRole::User.can_manage(UserRole::Owner));
    }
}
