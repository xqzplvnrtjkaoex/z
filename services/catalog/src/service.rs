use madome_proto::catalog::catalog_service_server::CatalogService;
use tonic::{Request, Response, Status};

pub struct CatalogServiceImpl;

#[tonic::async_trait]
impl CatalogService for CatalogServiceImpl {
    async fn health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}
