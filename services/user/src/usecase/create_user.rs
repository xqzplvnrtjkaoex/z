use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    domain::{
        error::user_error::UserError,
        ports::{UserPorts, user_repository::UserRepository},
        types::{role::UserRole, user::User},
    },
    payload::user::CreateUserPayload,
};

#[tracing::instrument(skip_all, fields(handle = %payload.handle()), err)]
pub async fn create_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: CreateUserPayload,
) -> Result<User, UserError> {
    payload.validate()?;

    if payload.role() == UserRole::Owner {
        return Err(UserError::OwnerRoleRejected);
    }

    let (handle, name, role) = payload.into_parts();

    let now = Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        handle,
        name,
        role,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    let saved = ctx.user_repo().save(&user).await?;

    tracing::info!(
        event = "user.created",
        user_id = %saved.id,
        handle = %saved.handle,
    );

    Ok(saved)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::domain::{
        error::repository_error::RepositoryError,
        ports::{
            UserPorts,
            user_repository::{MockUserRepository, UserRepository},
        },
    };

    struct TestContext {
        user_repo: MockUserRepository,
    }

    impl UserPorts for TestContext {
        fn user_repo(&self) -> &impl UserRepository {
            &self.user_repo
        }
    }

    #[tokio::test]
    async fn should_create_user_with_valid_handle_name_and_role_user() {
        let mut mock = MockUserRepository::new();
        mock.expect_save().once().returning(|u| Ok(u.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload::new(
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::User,
        );

        let result = create_user(&ctx, payload).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.handle, "testuser");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.role, UserRole::User);
        assert!(user.is_active);
    }

    #[tokio::test]
    async fn should_reject_owner_role_with_owner_role_rejected_error() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload::new(
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::Owner,
        );

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::OwnerRoleRejected)));
    }

    #[tokio::test]
    async fn should_reject_invalid_handle() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload::new(
            "ab".to_string(), // too short
            "Test User".to_string(),
            UserRole::User,
        );

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn should_reject_reserved_handle() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload =
            CreateUserPayload::new("admin".to_string(), "Test User".to_string(), UserRole::User);

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn should_reject_invalid_name() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload::new(
            "testuser".to_string(),
            "".to_string(), // empty name
            UserRole::User,
        );

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn should_return_handle_taken_when_handle_already_exists() {
        let mut mock = MockUserRepository::new();
        mock.expect_save()
            .once()
            .returning(|_| Err(RepositoryError::UniqueViolation("handle".to_string())));

        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload::new(
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::User,
        );

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::HandleTaken)));
    }
}
