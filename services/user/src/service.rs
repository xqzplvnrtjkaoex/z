use madome_proto::user::user_service_server::UserService;
use tonic::{Request, Response, Status};

pub struct UserServiceImpl;

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}
