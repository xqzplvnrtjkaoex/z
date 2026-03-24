use chrono::Utc;

use crate::{
    domain::{
        error::user_error::UserError,
        ports::{UserPorts, user_repository::UserRepository},
        types::user::User,
    },
    payload::user::ActivateUserPayload,
};

#[tracing::instrument(skip_all, fields(target_id = %payload.target_id(), actor_id = %payload.caller_id()), err)]
pub async fn activate_user(
    ctx: &(impl UserPorts + ?Sized),
    payload: ActivateUserPayload,
) -> Result<User, UserError> {
    if payload.caller_id() == payload.target_id() {
        return Err(UserError::SelfModification);
    }

    let mut target = ctx
        .user_repo()
        .find_by_id(payload.target_id())
        .await?
        .ok_or(UserError::UserNotFound)?;

    if !payload.caller_role().can_manage(target.role) {
        return Err(UserError::InsufficientRole {
            reason: format!(
                "role {} cannot manage role {}",
                payload.caller_role(),
                target.role
            ),
        });
    }

    if target.is_active {
        return Err(UserError::UserAlreadyActive);
    }

    target.is_active = true;
    target.updated_at = Utc::now();

    let updated = ctx.user_repo().update(&target).await?;

    tracing::info!(
        event = "user.activated",
        actor_id = %payload.caller_id(),
        target_id = %payload.target_id(),
    );

    Ok(updated)
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
    async fn should_activate_user_when_caller_has_higher_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::User, false);
        let target_clone = target.clone();
        let mut activated = target.clone();
        activated.is_active = true;
        let activated_clone = activated.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));
        mock.expect_update()
            .once()
            .returning(move |_| Ok(activated_clone.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = ActivateUserPayload::new(target_id, caller_id, UserRole::Admin);

        let result = activate_user(&ctx, payload).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_active);
    }

    #[tokio::test]
    async fn should_reject_self_activation() {
        let id = Uuid::new_v4();
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = ActivateUserPayload::new(id, id, UserRole::Admin);

        let result = activate_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::SelfModification)));
    }

    #[tokio::test]
    async fn should_return_insufficient_role_for_activate_when_caller_has_equal_or_lower_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::Admin, false);
        let target_clone = target.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = ActivateUserPayload::new(target_id, caller_id, UserRole::Admin);

        let result = activate_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InsufficientRole { .. })));
    }

    #[tokio::test]
    async fn should_return_user_already_active_when_user_is_already_active() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::User, true);
        let target_clone = target.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload = ActivateUserPayload::new(target_id, caller_id, UserRole::Admin);

        let result = activate_user(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::UserAlreadyActive)));
    }
}
