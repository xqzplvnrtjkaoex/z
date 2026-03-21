use madome_proto::catalog::catalog_service_server::CatalogServiceServer;
use tonic::transport::Server;

mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("catalog");

    let addr = madome_common::env::required_env("CATALOG_LISTEN_ADDR")
        .parse()
        .expect("invalid CATALOG_LISTEN_ADDR");

    tracing::info!(%addr, "catalog service starting");

    Server::builder()
        .add_service(CatalogServiceServer::new(service::CatalogServiceImpl))
        .serve(addr)
        .await?;

    Ok(())
}
