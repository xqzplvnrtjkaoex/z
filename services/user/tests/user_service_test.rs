use std::net::SocketAddr;
use std::time::Duration;

use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::sync::OnceCell;
use tonic::transport::{Endpoint, Server};

use madome_proto::user::user_service_client::UserServiceClient;
use madome_proto::user::user_service_server::UserServiceServer;
use madome_proto::user::{
    ActivateUserRequest, ChangeRoleRequest, CreateUserRequest, DeactivateUserRequest,
    GetUserRequest, ListUsersRequest, Role,
};
use user::adapter::context::UserContext;
use user::adapter::postgres::user_repository::PostgresUserRepository;
use user::app::UserHandler;

/// Stores the container (to keep it alive) and the connection URL.
struct TestContainer {
    _container: testcontainers::ContainerAsync<Postgres>,
    url: String,
}

static TEST_CONTAINER: OnceCell<TestContainer> = OnceCell::const_new();

/// Initializes the shared container and returns a fresh `DatabaseConnection`.
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

            // Run migrations once
            {
                let setup_db = Database::connect(&url)
                    .await
                    .expect("failed to connect for migration");
                user::migration::Migrator::up(&setup_db, None)
                    .await
                    .expect("failed to run migrations");
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

/// Find an available port by binding then dropping the listener.
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Start UserHandler gRPC service and return a connected client.
async fn start_service() -> UserServiceClient<tonic::transport::Channel> {
    let db = make_db().await;
    let repo = PostgresUserRepository::new(db);
    let ctx = UserContext::new(repo);
    let handler = UserHandler::new(ctx);

    let port = free_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    tokio::spawn(async move {
        Server::builder()
            .add_service(UserServiceServer::new(handler))
            .serve(addr)
            .await
            .expect("gRPC server failed");
    });

    // Small delay to allow server to bind and start accepting connections
    tokio::time::sleep(Duration::from_millis(100)).await;

    let channel = Endpoint::from_shared(format!("http://127.0.0.1:{port}"))
        .expect("invalid endpoint")
        .connect()
        .await
        .expect("failed to connect to gRPC server");

    UserServiceClient::new(channel)
}

#[tokio::test]
async fn should_create_and_get_user_via_grpc() {
    let mut client = start_service().await;

    let response = client
        .create_user(CreateUserRequest {
            handle: "svc_test1".to_string(),
            name: "Service Test".to_string(),
            role: Role::User as i32,
        })
        .await
        .expect("create_user failed");

    let user = response.into_inner();
    assert_eq!(user.handle, "svc_test1");
    assert_eq!(user.name, "Service Test");
    assert_eq!(user.role, Role::User as i32);
    assert!(user.is_active);
    assert!(!user.id.is_empty());

    // Get by ID
    let get_response = client
        .get_user(GetUserRequest {
            id: user.id.clone(),
        })
        .await
        .expect("get_user failed");
    assert_eq!(get_response.into_inner().handle, "svc_test1");
}

#[tokio::test]
async fn should_reject_owner_role_creation_via_grpc() {
    let mut client = start_service().await;

    let result = client
        .create_user(CreateUserRequest {
            handle: "svc_test2".to_string(),
            name: "Owner Test".to_string(),
            role: Role::Owner as i32,
        })
        .await;

    assert!(
        result.is_err(),
        "expected error when creating owner via API"
    );
    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "expected InvalidArgument, got {:?}",
        status.code()
    );
}

#[tokio::test]
async fn should_change_role_with_caller_context_via_grpc() {
    let mut client = start_service().await;

    // Create a user to be promoted
    let user = client
        .create_user(CreateUserRequest {
            handle: "svc_test3".to_string(),
            name: "Target".to_string(),
            role: Role::User as i32,
        })
        .await
        .expect("create_user failed")
        .into_inner();

    // Change role from user -> admin, with owner caller context
    let owner_id = uuid::Uuid::new_v4().to_string();
    let mut request = tonic::Request::new(ChangeRoleRequest {
        id: user.id.clone(),
        new_role: Role::Admin as i32,
    });
    request
        .metadata_mut()
        .insert("x-caller-id", owner_id.parse().unwrap());
    request
        .metadata_mut()
        .insert("x-caller-role", "owner".parse().unwrap());

    let response = client
        .change_role(request)
        .await
        .expect("change_role failed");
    assert_eq!(
        response.into_inner().role,
        Role::Admin as i32,
        "role should be updated to admin"
    );
}

#[tokio::test]
async fn should_deactivate_and_activate_user_via_grpc() {
    let mut client = start_service().await;

    let user = client
        .create_user(CreateUserRequest {
            handle: "svc_test4".to_string(),
            name: "Lifecycle".to_string(),
            role: Role::User as i32,
        })
        .await
        .expect("create_user failed")
        .into_inner();

    let admin_id = uuid::Uuid::new_v4().to_string();

    // Deactivate
    let mut deactivate_req = tonic::Request::new(DeactivateUserRequest {
        id: user.id.clone(),
    });
    deactivate_req
        .metadata_mut()
        .insert("x-caller-id", admin_id.parse().unwrap());
    deactivate_req
        .metadata_mut()
        .insert("x-caller-role", "owner".parse().unwrap());

    let deactivated = client
        .deactivate_user(deactivate_req)
        .await
        .expect("deactivate_user failed")
        .into_inner();
    assert!(!deactivated.is_active, "user should be deactivated");

    // Activate
    let mut activate_req = tonic::Request::new(ActivateUserRequest {
        id: user.id.clone(),
    });
    activate_req
        .metadata_mut()
        .insert("x-caller-id", admin_id.parse().unwrap());
    activate_req
        .metadata_mut()
        .insert("x-caller-role", "owner".parse().unwrap());

    let activated = client
        .activate_user(activate_req)
        .await
        .expect("activate_user failed")
        .into_inner();
    assert!(activated.is_active, "user should be reactivated");
}

#[tokio::test]
async fn should_list_users_with_pagination_via_grpc() {
    let mut client = start_service().await;

    // Create 3 users with unique handles
    for i in 0..3 {
        client
            .create_user(CreateUserRequest {
                handle: format!("svc_list_{i}"),
                name: format!("List User {i}"),
                role: Role::User as i32,
            })
            .await
            .expect("create_user failed");
    }

    let response = client
        .list_users(ListUsersRequest {
            limit: 2,
            cursor: None,
            include_inactive: false,
        })
        .await
        .expect("list_users failed");

    let list = response.into_inner();
    assert!(
        list.users.len() <= 2,
        "expected at most 2 results, got {}",
        list.users.len()
    );
    assert!(!list.users.is_empty(), "expected at least 1 user");
}

#[tokio::test]
async fn should_return_not_found_for_nonexistent_user_via_grpc() {
    let mut client = start_service().await;

    let result = client
        .get_user(GetUserRequest {
            id: uuid::Uuid::new_v4().to_string(),
        })
        .await;

    assert!(result.is_err(), "expected error for nonexistent user");
    assert_eq!(
        result.unwrap_err().code(),
        tonic::Code::NotFound,
        "expected NotFound status"
    );
}
