use madome_proto::auth::auth_service_server::AuthServiceServer;
use tonic::transport::Server;

mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("auth");

    let addr = madome_common::env::required_env("AUTH_LISTEN_ADDR")
        .parse()
        .expect("invalid AUTH_LISTEN_ADDR");

    tracing::info!(%addr, "auth service starting");

    Server::builder()
        .add_service(AuthServiceServer::new(service::AuthServiceImpl))
        .serve(addr)
        .await?;

    Ok(())
}
