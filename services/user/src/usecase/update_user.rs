use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::domain::{
    error::user_error::UserError,
    input::{handle_input::HandleInput, name_input::NameInput},
    ports::{UserPorts, user_repository::UserRepository},
    types::user::User,
};

pub struct UpdateUserPayload {
    pub id: Uuid,
    pub handle: Option<String>,
    pub name: Option<String>,
}

#[tracing::instrument(skip_all, fields(user_id = %payload.id), err)]
pub async fn update_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: UpdateUserPayload,
) -> Result<User, UserError> {
    let mut user = ctx
        .user_repo()
        .find_by_id(payload.id)
        .await?
        .ok_or(UserError::UserNotFound)?;

    if let Some(handle) = payload.handle {
        let handle_input = HandleInput::new(&handle);
        handle_input.validate_handle().map_err(|msg| {
            if msg.contains("reserved") {
                UserError::HandleReserved
            } else {
                UserError::InvalidHandle(msg)
            }
        })?;

        // Check handle uniqueness: if another user already has this handle
        if let Some(existing) = ctx.user_repo().find_by_handle(&handle).await?
            && existing.id != user.id
        {
            return Err(UserError::HandleTaken);
        }

        user.handle = handle;
    }

    if let Some(name) = payload.name {
        let name_input = NameInput::new(&name);
        name_input
            .validate()
            .map_err(|e| UserError::InvalidName(e.to_string()))?;
        user.name = name;
    }

    user.updated_at = Utc::now();
    ctx.user_repo().update(&user).await.map_err(UserError::from)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

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
        let payload = UpdateUserPayload {
            id,
            handle: Some("newhandle".to_string()),
            name: None,
        };

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
        let payload = UpdateUserPayload {
            id,
            handle: Some("takenhandle".to_string()),
            name: None,
        };

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
        // find_by_handle returns the same user (same id), so no conflict
        mock.expect_find_by_handle()
            .once()
            .returning(move |_| Ok(Some(user_clone2.clone())));
        mock.expect_update().once().returning(|u| Ok(u.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = UpdateUserPayload {
            id,
            handle: Some("samehandle".to_string()),
            name: None,
        };

        let result = update_user(&ctx, payload).await;
        assert!(result.is_ok());
    }
}
