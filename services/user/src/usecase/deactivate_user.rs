use chrono::Utc;
use uuid::Uuid;

use crate::domain::error::user_error::UserError;
use crate::domain::ports::UserPorts;
use crate::domain::ports::user_repository::UserRepository;
use crate::domain::types::role::UserRole;
use crate::domain::types::user::User;

pub struct DeactivateUserPayload {
    pub target_id: Uuid,
    pub caller_id: Uuid,
    pub caller_role: UserRole,
}

#[tracing::instrument(skip_all, fields(target_id = %payload.target_id, actor_id = %payload.caller_id), err)]
pub async fn deactivate_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: DeactivateUserPayload,
) -> Result<User, UserError> {
    // D-20: Self-modification blocked
    if payload.caller_id == payload.target_id {
        return Err(UserError::SelfModification);
    }

    let mut target = ctx
        .user_repo()
        .find_by_id(payload.target_id)
        .await?
        .ok_or(UserError::UserNotFound)?;

    // D-19: Role hierarchy check
    if !payload.caller_role.can_manage(target.role) {
        return Err(UserError::InsufficientRole {
            reason: format!(
                "role {} cannot manage role {}",
                payload.caller_role, target.role
            ),
        });
    }

    if !target.is_active {
        return Err(UserError::UserAlreadyInactive);
    }

    target.is_active = false;
    target.updated_at = Utc::now();

    let updated = ctx.user_repo().update(&target).await?;

    // D-58: Structured tracing for audit
    tracing::info!(
        event = "user.deactivated",
        actor_id = %payload.caller_id,
        target_id = %payload.target_id,
    );

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::UserPorts;
    use crate::domain::ports::user_repository::{MockUserRepository, UserRepository};
    use chrono::Utc;

    struct TestContext {
        user_repo: MockUserRepository,
    }

    impl UserPorts for TestContext {
        fn user_repo(&self) -> &impl UserRepository {
            &self.user_repo
        }
    }

    fn make_user(id: Uuid, role: UserRole, is_active: bool) -> User {
        let now = Utc::now();
        User {
            id,
            handle: "testuser".to_string(),
            name: "Test User".to_string(),
            role,
            is_active,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn should_deactivate_user_when_caller_has_higher_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::User, true);
        let target_clone = target.clone();
        let mut deactivated = target.clone();
        deactivated.is_active = false;
        let deactivated_clone = deactivated.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));
        mock.expect_update()
            .once()
            .returning(move |_| Ok(deactivated_clone.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = DeactivateUserPayload {
            target_id,
            caller_id,
            caller_role: UserRole::Admin,
        };

        let result = deactivate_user(&ctx, payload).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_active);
    }

    #[tokio::test]
    async fn should_reject_self_deactivation() {
        let id = Uuid::new_v4();
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = DeactivateUserPayload {
            target_id: id,
            caller_id: id, // same as target
            caller_role: UserRole::Admin,
        };

        let result = deactivate_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::SelfModification)));
    }

    #[tokio::test]
    async fn should_return_insufficient_role_when_caller_has_lower_or_equal_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::Admin, true);
        let target_clone = target.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = DeactivateUserPayload {
            target_id,
            caller_id,
            caller_role: UserRole::Admin, // equal role, not higher
        };

        let result = deactivate_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InsufficientRole { .. })));
    }

    #[tokio::test]
    async fn should_return_user_already_inactive_when_user_is_already_deactivated() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::User, false); // already inactive
        let target_clone = target.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = DeactivateUserPayload {
            target_id,
            caller_id,
            caller_role: UserRole::Admin,
        };

        let result = deactivate_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::UserAlreadyInactive)));
    }
}
