use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{error::repository_error::RepositoryError, types::user::User};

#[cfg_attr(test, mockall::automock(target = UserRepository))]
#[trait_variant::make(UserRepository: Send)]
pub trait LocalUserRepository {
    async fn save(&self, user: &User) -> Result<User, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError>;
    async fn find_by_handle(&self, handle: &str) -> Result<Option<User>, RepositoryError>;
    async fn list(
        &self,
        limit: u64,
        cursor: Option<(DateTime<Utc>, Uuid)>,
        include_inactive: bool,
    ) -> Result<Vec<User>, RepositoryError>;
    async fn update(&self, user: &User) -> Result<User, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::domain::types::role::UserRole;

    // Test that MockUserRepository compiles and can set expectations.
    // This validates that #[automock(target = UserRepository)] generates
    // a mock that implements the Send variant.
    #[tokio::test]
    async fn should_compile_and_use_mock_user_repository() {
        let mut mock = MockUserRepository::new();
        let test_id = Uuid::new_v4();
        let now = Utc::now();

        let expected_user = User {
            id: test_id,
            handle: "testuser".to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        let expected_clone = expected_user.clone();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(expected_clone.clone())));

        let result = UserRepository::find_by_id(&mock, test_id).await;
        assert!(result.is_ok());
        let user = result.unwrap().unwrap();
        assert_eq!(user.id, test_id);
        assert_eq!(user.handle, "testuser");
    }
}
