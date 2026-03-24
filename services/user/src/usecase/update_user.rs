use chrono::Utc;
use validator::Validate;

use crate::{
    domain::{
        error::user_error::UserError,
        ports::{UserPorts, user_repository::UserRepository},
        types::user::User,
    },
    payload::user::UpdateUserPayload,
};

#[tracing::instrument(skip_all, fields(user_id = %payload.id()), err)]
pub async fn update_user(
    ctx: &(impl UserPorts + ?Sized),
    mut payload: UpdateUserPayload,
) -> Result<User, UserError> {
    payload.validate()?;

    let mut user = ctx
        .user_repo()
        .find_by_id(payload.id())
        .await?
        .ok_or(UserError::UserNotFound)?;

    if let Some(handle) = payload.take_handle() {
        if let Some(existing) = ctx.user_repo().find_by_handle(&handle).await?
            && existing.id != user.id
        {
            return Err(UserError::HandleTaken);
        }

        user.handle = handle;
    }

    if let Some(name) = payload.take_name() {
        user.name = name;
    }

    user.updated_at = Utc::now();
    ctx.user_repo().update(&user).await.map_err(UserError::from)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::domain::{
        ports::{
            UserPorts,
            user_repository::{MockUserRepository, UserRepository},
        },
        types::role::UserRole,
    };

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
    async fn should_update_user_handle_when_new_handle_is_unique() {
        let id = Uuid::new_v4();
        let user = make_user(id, "oldhandle");
        let user_clone = user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(user_clone.clone())));
        mock.expect_find_by_handle().once().returning(|_| Ok(None));
        mock.expect_update().once().returning(|u| Ok(u.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = UpdateUserPayload::new(id, Some("newhandle".to_string()), None);

        let result = update_user(&ctx, payload).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().handle, "newhandle");
    }

    #[tokio::test]
    async fn should_return_handle_taken_when_new_handle_already_used_by_different_user() {
        let id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let user = make_user(id, "myhandle");
        let other_user = make_user(other_id, "takenhandle");
        let user_clone = user.clone();
        let other_clone = other_user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(user_clone.clone())));
        mock.expect_find_by_handle()
            .once()
            .returning(move |_| Ok(Some(other_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = UpdateUserPayload::new(id, Some("takenhandle".to_string()), None);

        let result = update_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::HandleTaken)));
    }

    #[tokio::test]
    async fn should_allow_user_to_update_to_same_handle() {
        let id = Uuid::new_v4();
        let user = make_user(id, "samehandle");
        let user_clone = user.clone();
        let user_clone2 = user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(user_clone.clone())));
        mock.expect_find_by_handle()
            .once()
            .returning(move |_| Ok(Some(user_clone2.clone())));
        mock.expect_update().once().returning(|u| Ok(u.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = UpdateUserPayload::new(id, Some("samehandle".to_string()), None);

        let result = update_user(&ctx, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_invalid_handle_on_update() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = UpdateUserPayload::new(Uuid::new_v4(), Some("ab".to_string()), None);

        let result = update_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InvalidInput(_))));
    }
}
