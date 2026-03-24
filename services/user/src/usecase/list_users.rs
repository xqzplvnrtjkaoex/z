use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use validator::Validate;

use crate::{
    domain::{
        error::user_error::UserError,
        ports::{UserPorts, user_repository::UserRepository},
        types::user::User,
    },
    payload::user::ListUsersPayload,
};

#[tracing::instrument(skip_all, err)]
pub async fn list_users(
    ctx: &(impl UserPorts + ?Sized),
    payload: ListUsersPayload,
) -> Result<(Vec<User>, Option<String>), UserError> {
    payload.validate()?;
    let limit = payload.limit();
    let cursor = payload.cursor()?;

    // Fetch limit + 1 to detect if there are more results
    let mut users = ctx
        .user_repo()
        .list(limit + 1, cursor, payload.include_inactive())
        .await?;

    let next_cursor = if users.len() > limit as usize {
        users.pop(); // remove the extra item
        let last = users.last().unwrap();
        let cursor_str = format!("{},{}", last.created_at.to_rfc3339(), last.id);
        Some(BASE64.encode(cursor_str.as_bytes()))
    } else {
        None
    };

    Ok((users, next_cursor))
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

    fn make_user(handle: &str) -> User {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            handle: handle.to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn should_return_users_with_no_next_cursor_when_under_limit() {
        let users = vec![make_user("user1"), make_user("user2")];
        let users_clone = users.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_list()
            .once()
            .returning(move |_, _, _| Ok(users_clone.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = ListUsersPayload {
            limit: 25,
            cursor: None,
            include_inactive: false,
        };

        let result = list_users(&ctx, payload).await;
        assert!(result.is_ok());
        let (returned_users, next_cursor) = result.unwrap();
        assert_eq!(returned_users.len(), 2);
        assert!(next_cursor.is_none());
    }

    #[tokio::test]
    async fn should_return_next_cursor_when_more_results_exist() {
        let users: Vec<User> = (0..26).map(|i| make_user(&format!("user{i}"))).collect();
        let users_clone = users.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_list()
            .once()
            .returning(move |_, _, _| Ok(users_clone.clone()));

        let ctx = TestContext { user_repo: mock };
        let payload = ListUsersPayload {
            limit: 25,
            cursor: None,
            include_inactive: false,
        };

        let result = list_users(&ctx, payload).await;
        assert!(result.is_ok());
        let (returned_users, next_cursor) = result.unwrap();
        assert_eq!(returned_users.len(), 25);
        assert!(next_cursor.is_some());
    }

    #[tokio::test]
    async fn should_use_default_limit_25_when_limit_is_zero() {
        let mut mock = MockUserRepository::new();
        mock.expect_list()
            .once()
            .withf(|limit, _, _| *limit == 26) // 25 + 1 for has-more detection
            .returning(|_, _, _| Ok(vec![]));

        let ctx = TestContext { user_repo: mock };
        let payload = ListUsersPayload {
            limit: 0,
            cursor: None,
            include_inactive: false,
        };

        let result = list_users(&ctx, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_limit_over_100() {
        let mock = MockUserRepository::new();
        let ctx = TestContext { user_repo: mock };
        let payload = ListUsersPayload {
            limit: 101,
            cursor: None,
            include_inactive: false,
        };

        let result = list_users(&ctx, payload).await;
        assert!(matches!(result, Err(UserError::InvalidInput(_))));
    }
}
