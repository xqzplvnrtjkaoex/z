use madome_proto::auth::auth_service_client::AuthServiceClient;
use madome_proto::catalog::catalog_service_client::CatalogServiceClient;
use madome_proto::user::user_service_client::UserServiceClient;
use tower_http::trace::TraceLayer;

pub mod routes;
pub mod state;

use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("gateway");

    let auth_addr = madome_common::env::required_env("AUTH_GRPC_ADDR");
    let catalog_addr = madome_common::env::required_env("CATALOG_GRPC_ADDR");
    let user_addr = madome_common::env::required_env("USER_GRPC_ADDR");
    let gateway_addr = madome_common::env::optional_env("GATEWAY_ADDR", "0.0.0.0:3000");

    tracing::info!("connecting to auth at {auth_addr}");
    let auth_client = AuthServiceClient::connect(auth_addr).await?;

    tracing::info!("connecting to catalog at {catalog_addr}");
    let catalog_client = CatalogServiceClient::connect(catalog_addr).await?;

    tracing::info!("connecting to user at {user_addr}");
    let user_client = UserServiceClient::connect(user_addr).await?;

    let app_state = AppState {
        auth_client,
        catalog_client,
        user_client,
    };

    let app = routes::create_router(app_state).layer(TraceLayer::new_for_http());

    let addr: std::net::SocketAddr = gateway_addr.parse()?;
    tracing::info!(%addr, "gateway starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
