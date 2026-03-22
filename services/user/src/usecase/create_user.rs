use chrono::Utc;
use uuid::Uuid;

use crate::domain::error::user_error::UserError;
use crate::domain::ports::UserPorts;
use crate::domain::ports::user_repository::UserRepository;
use crate::domain::types::role::UserRole;
use crate::domain::types::user::User;
use crate::domain::types::validation::{HandleInput, NameInput};

pub struct CreateUserPayload {
    pub handle: String,
    pub name: String,
    pub role: UserRole,
}

pub async fn create_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: CreateUserPayload,
) -> Result<User, UserError> {
    // D-60: Reject owner role assignment via API
    if payload.role == UserRole::Owner {
        return Err(UserError::OwnerRoleRejected);
    }

    // D-12: Validate handle via HandleInput struct
    let handle_input = HandleInput::new(&payload.handle);
    handle_input.validate_handle().map_err(|msg| {
        if msg.contains("reserved") {
            UserError::HandleReserved
        } else {
            UserError::InvalidHandle(msg)
        }
    })?;

    // D-12: Validate name via NameInput struct
    let name_input = NameInput::new(&payload.name);
    name_input.validate_name().map_err(UserError::InvalidName)?;

    let now = Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        handle: payload.handle,
        name: payload.name,
        role: payload.role,
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
    use super::*;
    use crate::domain::error::repository_error::RepositoryError;
    use crate::domain::ports::UserPorts;
    use crate::domain::ports::user_repository::{MockUserRepository, UserRepository};

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
        let payload = CreateUserPayload {
            handle: "testuser".to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
        };

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
        let payload = CreateUserPayload {
            handle: "testuser".to_string(),
            name: "Test User".to_string(),
            role: UserRole::Owner,
        };

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::OwnerRoleRejected)));
    }

    #[tokio::test]
    async fn should_reject_invalid_handle_with_invalid_handle_error() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload {
            handle: "ab".to_string(), // too short
            name: "Test User".to_string(),
            role: UserRole::User,
        };

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InvalidHandle(_))));
    }

    #[tokio::test]
    async fn should_reject_reserved_handle_with_handle_reserved_error() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload {
            handle: "admin".to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
        };

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::HandleReserved)));
    }

    #[tokio::test]
    async fn should_reject_invalid_name_with_invalid_name_error() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload {
            handle: "testuser".to_string(),
            name: "".to_string(), // empty name
            role: UserRole::User,
        };

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InvalidName(_))));
    }

    #[tokio::test]
    async fn should_return_handle_taken_when_handle_already_exists() {
        let mut mock = MockUserRepository::new();
        mock.expect_save()
            .once()
            .returning(|_| Err(RepositoryError::UniqueViolation("handle".to_string())));

        let ctx = TestContext { user_repo: mock };
        let payload = CreateUserPayload {
            handle: "testuser".to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
        };

        let result = create_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::HandleTaken)));
    }
}
