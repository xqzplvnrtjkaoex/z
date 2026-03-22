use uuid::Uuid;

use crate::domain::error::user_error::UserError;
use crate::domain::ports::UserPorts;
use crate::domain::ports::user_repository::UserRepository;
use crate::domain::types::user::User;

pub async fn get_user(ctx: &(impl UserPorts + ?Sized), id: Uuid) -> Result<User, UserError> {
    // D-53: Always return user regardless of is_active
    ctx.user_repo()
        .find_by_id(id)
        .await?
        .ok_or(UserError::UserNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::UserPorts;
    use crate::domain::ports::user_repository::{MockUserRepository, UserRepository};
    use crate::domain::types::role::UserRole;
    use chrono::Utc;

    struct TestContext {
        user_repo: MockUserRepository,
    }

    impl UserPorts for TestContext {
        fn user_repo(&self) -> &impl UserRepository {
            &self.user_repo
        }
    }

    fn make_user(id: Uuid, handle: &str) -> User {
        let now = Utc::now();
        User {
            id,
            handle: handle.to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn should_return_user_when_found_by_id() {
        let id = Uuid::new_v4();
        let user = make_user(id, "testuser");
        let user_clone = user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(user_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let result = get_user(&ctx, id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, id);
    }

    #[tokio::test]
    async fn should_return_user_not_found_when_id_does_not_exist() {
        let id = Uuid::new_v4();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id().once().returning(|_| Ok(None));

        let ctx = TestContext { user_repo: mock };
        let result = get_user(&ctx, id).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));
    }
}
