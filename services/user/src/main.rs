use madome_proto::user::user_service_server::UserServiceServer;
use tonic::transport::Server;

mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("user");

    let addr = madome_common::env::required_env("USER_LISTEN_ADDR")
        .parse()
        .expect("invalid USER_LISTEN_ADDR");

    tracing::info!(%addr, "user service starting");

    Server::builder()
        .add_service(UserServiceServer::new(service::UserServiceImpl))
        .serve(addr)
        .await?;

    Ok(())
}
