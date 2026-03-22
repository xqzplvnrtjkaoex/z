use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::error::user_error::UserError;
use crate::domain::ports::UserPorts;
use crate::domain::ports::user_repository::UserRepository;
use crate::domain::types::user::User;

pub struct ListUsersPayload {
    pub limit: u64,
    pub cursor: Option<String>,
    pub include_inactive: bool,
}

#[tracing::instrument(skip_all, err)]
pub async fn list_users(
    ctx: &(impl UserPorts + ?Sized),
    payload: ListUsersPayload,
) -> Result<(Vec<User>, Option<String>), UserError> {
    // Cap and default limit
    let limit = if payload.limit == 0 {
        25
    } else if payload.limit > 100 {
        100
    } else {
        payload.limit
    };

    // Decode cursor if present
    let decoded_cursor = if let Some(cursor_str) = payload.cursor {
        let bytes = BASE64
            .decode(cursor_str.as_bytes())
            .map_err(|_| UserError::Internal("invalid cursor encoding".to_string()))?;
        let s = String::from_utf8(bytes)
            .map_err(|_| UserError::Internal("invalid cursor encoding".to_string()))?;
        let (ts_part, uuid_part) = s
            .rsplit_once(',')
            .ok_or_else(|| UserError::Internal("invalid cursor format".to_string()))?;
        let ts = ts_part
            .parse::<DateTime<Utc>>()
            .map_err(|_| UserError::Internal("invalid cursor timestamp".to_string()))?;
        let id = uuid_part
            .parse::<Uuid>()
            .map_err(|_| UserError::Internal("invalid cursor uuid".to_string()))?;
        Some((ts, id))
    } else {
        None
    };

    // Fetch limit + 1 to detect if there are more results
    let mut users = ctx
        .user_repo()
        .list(limit + 1, decoded_cursor, payload.include_inactive)
        .await?;

    // Determine next cursor
    let next_cursor = if users.len() > limit as usize {
        users.pop(); // remove the extra item
        // Encode last item's (created_at, id) as cursor
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
        // Build 26 users (limit is 25, so 26 signals more pages)
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
}
