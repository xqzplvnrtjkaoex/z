use chrono::Utc;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::sync::OnceCell;
use uuid::Uuid;

use user::adapter::postgres::user_repository::PostgresUserRepository;
use user::domain::error::repository_error::RepositoryError;
use user::domain::ports::user_repository::UserRepository;
use user::domain::types::role::UserRole;
use user::domain::types::user::User;

/// Stores the container (to keep it alive) and the connection URL so
/// each test can open its own connection pool against the same container.
struct TestContainer {
    _container: testcontainers::ContainerAsync<Postgres>,
    url: String,
}

static TEST_CONTAINER: OnceCell<TestContainer> = OnceCell::const_new();

/// Initializes the shared container and returns a fresh `DatabaseConnection`.
/// The container is started only once; each call opens a new connection.
async fn make_db() -> DatabaseConnection {
    let tc = TEST_CONTAINER
        .get_or_init(|| async {
            let container = Postgres::default()
                .start()
                .await
                .expect("failed to start postgres container");

            let host = container.get_host().await.expect("container host");
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("container port");

            let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

            // Run migrations once on a temporary connection
            {
                let setup_db = Database::connect(&url)
                    .await
                    .expect("failed to connect for migration");
                user::migration::Migrator::up(&setup_db, None)
                    .await
                    .expect("failed to run migrations");
                // Drop the setup connection so its pool is released
            }

            TestContainer {
                _container: container,
                url,
            }
        })
        .await;

    Database::connect(&tc.url)
        .await
        .expect("failed to open test connection")
}

fn make_test_user(handle: &str, name: &str, role: UserRole) -> User {
    User {
        id: Uuid::new_v4(),
        handle: handle.to_string(),
        name: name.to_string(),
        role,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn should_save_and_find_user_by_id() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    let user = make_test_user("integ_test1", "Test User", UserRole::User);
    let saved = repo.save(&user).await.expect("save failed");
    assert_eq!(saved.handle, "integ_test1");

    let found = repo.find_by_id(saved.id).await.expect("find_by_id failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().handle, "integ_test1");
}

#[tokio::test]
async fn should_find_user_by_handle_case_insensitive() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    let user = make_test_user("CaseTest", "Case Test", UserRole::User);
    repo.save(&user).await.expect("save failed");

    // Find with lowercase variant
    let found = repo
        .find_by_handle("casetest")
        .await
        .expect("find_by_handle failed");
    assert!(found.is_some(), "should find user case-insensitively");
    assert_eq!(found.unwrap().handle, "CaseTest"); // original case preserved
}

#[tokio::test]
async fn should_reject_duplicate_handle_case_insensitive() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    let user1 = make_test_user("dupl_test", "User 1", UserRole::User);
    repo.save(&user1).await.expect("save user1 failed");

    let user2 = make_test_user("DUPL_TEST", "User 2", UserRole::User);
    let result = repo.save(&user2).await;
    assert!(
        matches!(result, Err(RepositoryError::UniqueViolation(_))),
        "expected UniqueViolation, got: {:?}",
        result
    );
}

#[tokio::test]
async fn should_list_users_with_pagination() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    // Create 3 users with unique handles
    for i in 0..3 {
        let user = make_test_user(
            &format!("page_test_{i}"),
            &format!("Page User {i}"),
            UserRole::User,
        );
        repo.save(&user).await.expect("save failed");
    }

    let page1 = repo.list(2, None, false).await.expect("list failed");
    // At most 2 results due to limit
    assert!(
        page1.len() <= 2,
        "expected at most 2 results, got {}",
        page1.len()
    );
    assert!(!page1.is_empty(), "expected at least 1 result");
}

#[tokio::test]
async fn should_list_active_users_only_by_default() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);

    // Save an active user then deactivate them
    let user = make_test_user("active_test", "Active Test", UserRole::User);
    let saved = repo.save(&user).await.expect("save failed");

    let deactivated = User {
        is_active: false,
        updated_at: Utc::now(),
        ..saved
    };
    repo.update(&deactivated).await.expect("update failed");

    // active only: should not find the deactivated user
    let active_only = repo.list(100, None, false).await.expect("list failed");
    assert!(
        !active_only.iter().any(|u| u.handle == "active_test"),
        "deactivated user should not appear in active-only listing"
    );

    // include inactive: should find the deactivated user
    let all = repo.list(100, None, true).await.expect("list failed");
    assert!(
        all.iter().any(|u| u.handle == "active_test"),
        "deactivated user should appear in include_inactive listing"
    );
}

#[tokio::test]
async fn should_update_user_handle_and_name() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    let user = make_test_user("upd_test1", "Old Name", UserRole::User);
    let saved = repo.save(&user).await.expect("save failed");

    let updated_user = User {
        handle: "upd_test2".to_string(),
        name: "New Name".to_string(),
        updated_at: Utc::now(),
        ..saved
    };
    let updated = repo.update(&updated_user).await.expect("update failed");
    assert_eq!(updated.handle, "upd_test2");
    assert_eq!(updated.name, "New Name");
}

#[tokio::test]
async fn should_return_none_for_nonexistent_user_id() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    let nonexistent_id = Uuid::new_v4();
    let found = repo
        .find_by_id(nonexistent_id)
        .await
        .expect("find_by_id failed");
    assert!(found.is_none(), "expected None for nonexistent user");
}

#[tokio::test]
async fn should_return_none_for_nonexistent_handle() {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    let found = repo
        .find_by_handle("definitely_nonexistent_handle_xyz")
        .await
        .expect("find_by_handle failed");
    assert!(found.is_none(), "expected None for nonexistent handle");
}
