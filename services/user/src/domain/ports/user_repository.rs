use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::error::repository_error::RepositoryError;
use crate::domain::types::user::User;

#[cfg_attr(test, mockall::automock)]
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
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::types::role::UserRole;

    // Test that MockLocalUserRepository compiles and can set expectations
    // This validates trait_variant + mockall compatibility (Test 13)
    #[tokio::test]
    async fn should_compile_and_use_mock_local_user_repository() {
        let mut mock = MockLocalUserRepository::new();
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

        let result = mock.find_by_id(test_id).await;
        assert!(result.is_ok());
        let user = result.unwrap().unwrap();
        assert_eq!(user.id, test_id);
        assert_eq!(user.handle, "testuser");
    }
}
