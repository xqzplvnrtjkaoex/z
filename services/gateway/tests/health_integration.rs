use gateway::routes::create_router;
use gateway::state::AppState;
use madome_proto::auth::auth_service_client::AuthServiceClient;
use madome_proto::auth::auth_service_server::{AuthService, AuthServiceServer};
use madome_proto::catalog::catalog_service_client::CatalogServiceClient;
use madome_proto::catalog::catalog_service_server::{CatalogService, CatalogServiceServer};
use madome_proto::user::user_service_client::UserServiceClient;
use madome_proto::user::user_service_server::{UserService, UserServiceServer};
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

// Test stub implementations

struct TestAuthService;
#[tonic::async_trait]
impl AuthService for TestAuthService {
    async fn health(&self, _req: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}

struct TestCatalogService;
#[tonic::async_trait]
impl CatalogService for TestCatalogService {
    async fn health(&self, _req: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}

struct TestUserService;
#[tonic::async_trait]
impl UserService for TestUserService {
    async fn health(&self, _req: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}

/// Find an available port by briefly binding and immediately releasing.
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Start an auth gRPC service on the given port.
async fn start_auth_grpc(port: u16) {
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    tokio::spawn(async move {
        Server::builder()
            .add_service(AuthServiceServer::new(TestAuthService))
            .serve(addr)
            .await
            .unwrap();
    });
    // Small delay to allow server to bind
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

async fn start_catalog_grpc(port: u16) {
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    tokio::spawn(async move {
        Server::builder()
            .add_service(CatalogServiceServer::new(TestCatalogService))
            .serve(addr)
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

async fn start_user_grpc(port: u16) {
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    tokio::spawn(async move {
        Server::builder()
            .add_service(UserServiceServer::new(TestUserService))
            .serve(addr)
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

/// Build gRPC clients using connect_lazy so gateway can start even if a service is down.
fn make_auth_client(port: u16) -> AuthServiceClient<tonic::transport::Channel> {
    let endpoint =
        tonic::transport::Endpoint::from_shared(format!("http://127.0.0.1:{port}")).unwrap();
    AuthServiceClient::new(endpoint.connect_lazy())
}

fn make_catalog_client(port: u16) -> CatalogServiceClient<tonic::transport::Channel> {
    let endpoint =
        tonic::transport::Endpoint::from_shared(format!("http://127.0.0.1:{port}")).unwrap();
    CatalogServiceClient::new(endpoint.connect_lazy())
}

fn make_user_client(port: u16) -> UserServiceClient<tonic::transport::Channel> {
    let endpoint =
        tonic::transport::Endpoint::from_shared(format!("http://127.0.0.1:{port}")).unwrap();
    UserServiceClient::new(endpoint.connect_lazy())
}

/// Start the gateway pointing to the given gRPC ports, return the HTTP port.
async fn start_gateway(auth_port: u16, catalog_port: u16, user_port: u16) -> u16 {
    let state = AppState {
        auth_client: make_auth_client(auth_port),
        catalog_client: make_catalog_client(catalog_port),
        user_client: make_user_client(user_port),
    };

    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    // Small delay to allow gateway to bind
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    port
}

#[tokio::test]
async fn gateway_own_health() {
    let auth_port = free_port();
    let catalog_port = free_port();
    let user_port = free_port();
    start_auth_grpc(auth_port).await;
    start_catalog_grpc(catalog_port).await;
    start_user_grpc(user_port).await;
    let gateway_port = start_gateway(auth_port, catalog_port, user_port).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{gateway_port}/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn service_health_all_ok() {
    let auth_port = free_port();
    let catalog_port = free_port();
    let user_port = free_port();
    start_auth_grpc(auth_port).await;
    start_catalog_grpc(catalog_port).await;
    start_user_grpc(user_port).await;
    let gateway_port = start_gateway(auth_port, catalog_port, user_port).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{gateway_port}/v1/health/services"
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["services"]["auth"], "ok");
    assert_eq!(body["services"]["catalog"], "ok");
    assert_eq!(body["services"]["user"], "ok");
    assert!(body["request_id"].as_str().is_some_and(|s| !s.is_empty()));
}

#[tokio::test]
async fn service_health_partial_failure() {
    // Only start catalog and user; auth port is deliberately unused
    let auth_port = free_port(); // port is allocated but no server starts on it
    let catalog_port = free_port();
    let user_port = free_port();
    // DO NOT start auth - we intentionally leave auth_port unbound
    start_catalog_grpc(catalog_port).await;
    start_user_grpc(user_port).await;
    let gateway_port = start_gateway(auth_port, catalog_port, user_port).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{gateway_port}/v1/health/services"
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["services"]["auth"], "unavailable");
    assert_eq!(body["services"]["catalog"], "ok");
    assert_eq!(body["services"]["user"], "ok");
}
