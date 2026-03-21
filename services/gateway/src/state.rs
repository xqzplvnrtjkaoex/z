use madome_proto::auth::auth_service_client::AuthServiceClient;
use madome_proto::catalog::catalog_service_client::CatalogServiceClient;
use madome_proto::user::user_service_client::UserServiceClient;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct AppState {
    pub auth_client: AuthServiceClient<Channel>,
    pub catalog_client: CatalogServiceClient<Channel>,
    pub user_client: UserServiceClient<Channel>,
}
