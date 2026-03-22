use madome_proto::user::{
    ActivateUserRequest, ChangeRoleRequest, CreateUserRequest, DeactivateUserRequest,
    GetUserByHandleRequest, GetUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest,
    UserResponse, user_service_server::UserService,
};
use tonic::{Request, Response, Status};

pub struct UserServiceImpl;

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn create_user(
        &self,
        _request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_user(
        &self,
        _request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_user_by_handle(
        &self,
        _request: Request<GetUserByHandleRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn update_user(
        &self,
        _request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn deactivate_user(
        &self,
        _request: Request<DeactivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn activate_user(
        &self,
        _request: Request<ActivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn change_role(
        &self,
        _request: Request<ChangeRoleRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}
