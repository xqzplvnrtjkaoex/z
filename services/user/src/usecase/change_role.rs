use chrono::Utc;

use crate::{
    domain::{
        error::user_error::UserError,
        ports::{UserPorts, user_repository::UserRepository},
        types::user::User,
    },
    payload::user::ChangeRolePayload,
};

#[tracing::instrument(skip_all, fields(target_id = %payload.target_id(), actor_id = %payload.caller_id()), err)]
pub async fn change_role(
    ctx: &(impl UserPorts + ?Sized),
    payload: ChangeRolePayload,
) -> Result<User, UserError> {
    if payload.new_role() == UserRole::Owner {
        return Err(UserError::OwnerRoleRejected);
    }

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
                "role {} cannot manage user with role {}",
                payload.caller_role(),
                target.role
            ),
        });
    }

    if !payload.caller_role().can_manage(payload.new_role()) {
        return Err(UserError::InsufficientRole {
            reason: format!(
                "role {} cannot assign role {}",
                payload.caller_role(),
                payload.new_role()
            ),
        });
    }

    let old_role = target.role;
    target.role = payload.new_role();
    target.updated_at = Utc::now();

    let updated = ctx.user_repo().update(&target).await?;

    tracing::info!(
        event = "user.role_changed",
        actor_id = %payload.caller_id(),
        target_id = %payload.target_id(),
        old_role = %old_role,
        new_role = %payload.new_role(),
    );

    Ok(updated)
}

use crate::domain::types::role::UserRole;

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

    fn make_user(id: Uuid, role: UserRole) -> User {
        let now = Utc::now();
        User {
            id,
            handle: "testuser".to_string(),
            name: "Test User".to_string(),
            role,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn should_change_role_when_caller_has_higher_role_than_both_target_and_new_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::User);
        let target_clone = target.clone();
        let mut updated = target.clone();
        updated.role = UserRole::Admin;
        let updated_clone = updated.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));
        mock.expect_update()
            .once()
            .returning(move |_| Ok(updated_clone.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload =
            ChangeRolePayload::new(target_id, UserRole::Admin, caller_id, UserRole::Owner);

        let result = change_role(&ctx, payload).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().role, UserRole::Admin);
    }

    #[tokio::test]
    async fn should_reject_self_role_change() {
        let id = Uuid::new_v4();
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = ChangeRolePayload::new(id, UserRole::User, id, UserRole::Owner);

        let result = change_role(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::SelfModification)));
    }

    #[tokio::test]
    async fn should_reject_assigning_owner_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload =
            ChangeRolePayload::new(target_id, UserRole::Owner, caller_id, UserRole::Owner);

        let result = change_role(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::OwnerRoleRejected)));
    }

    #[tokio::test]
    async fn should_return_insufficient_role_when_caller_cannot_manage_current_target_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::Admin);
        let target_clone = target.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload =
            ChangeRolePayload::new(target_id, UserRole::User, caller_id, UserRole::Admin);

        let result = change_role(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InsufficientRole { .. })));
    }

    #[tokio::test]
    async fn should_return_insufficient_role_when_caller_cannot_assign_new_role() {
        let caller_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let target = make_user(target_id, UserRole::User);
        let target_clone = target.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .once()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let ctx = TestContext { user_repo: mock };
        let payload =
            ChangeRolePayload::new(target_id, UserRole::Admin, caller_id, UserRole::Admin);

        let result = change_role(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InsufficientRole { .. })));
    }
}
