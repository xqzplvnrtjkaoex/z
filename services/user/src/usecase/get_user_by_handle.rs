use crate::{
    domain::{
        error::user_error::UserError,
        ports::{UserPorts, user_repository::UserRepository},
        types::{role::UserRole, user::User},
    },
    payload::user::GetUserByHandlePayload,
};

#[tracing::instrument(skip_all, fields(handle = %payload.handle()), err)]
pub async fn get_user_by_handle(
    ctx: &(impl UserPorts + ?Sized),
    payload: GetUserByHandlePayload,
) -> Result<User, UserError> {
    let user = ctx
        .user_repo()
        .find_by_handle(payload.handle())
        .await?
        .ok_or(UserError::UserNotFound)?;

    if !user.is_active && !payload.caller_role().can_manage(UserRole::User) {
        return Err(UserError::UserNotFound);
    }

    Ok(user)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::domain::ports::{
        UserPorts,
        user_repository::{MockUserRepository, UserRepository},
    };

    struct TestContext {
        user_repo: MockUserRepository,
    }

    impl UserPorts for TestContext {
        fn user_repo(&self) -> &impl UserRepository {
            &self.user_repo
        }
    }

    fn make_user(handle: &str, is_active: bool) -> User {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            handle: handle.to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
            is_active,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn should_return_user_when_handle_exists_and_active() {
        let user = make_user("testuser", true);
        let user_clone = user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_handle()
            .once()
            .returning(move |_| Ok(Some(user_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = GetUserByHandlePayload::new("testuser".to_string(), UserRole::User);

        let result = get_user_by_handle(&ctx, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_user_not_found_when_handle_does_not_exist() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_handle().once().returning(|_| Ok(None));

        let ctx = TestContext { user_repo: mock };
        let payload = GetUserByHandlePayload::new("nonexistent".to_string(), UserRole::User);

        let result = get_user_by_handle(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));
    }

    #[tokio::test]
    async fn should_hide_inactive_user_from_non_admin_caller() {
        let user = make_user("inactive_user", false);
        let user_clone = user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_handle()
            .once()
            .returning(move |_| Ok(Some(user_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = GetUserByHandlePayload::new("inactive_user".to_string(), UserRole::User);

        let result = get_user_by_handle(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));
    }

    #[tokio::test]
    async fn should_show_inactive_user_to_admin_caller() {
        let user = make_user("inactive_user", false);
        let user_clone = user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_handle()
            .once()
            .returning(move |_| Ok(Some(user_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = GetUserByHandlePayload::new("inactive_user".to_string(), UserRole::Admin);

        let result = get_user_by_handle(&ctx, payload).await;
        assert!(result.is_ok());
    }
}
