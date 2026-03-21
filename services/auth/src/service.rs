use madome_proto::auth::auth_service_server::AuthService;
use tonic::{Request, Response, Status};

pub struct AuthServiceImpl;

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}
