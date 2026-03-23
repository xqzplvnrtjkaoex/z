pub mod rpc;

use madome_proto::user::{
    ActivateUserRequest, ChangeRoleRequest, CreateUserRequest, DeactivateUserRequest,
    GetUserByHandleRequest, GetUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest,
    UserResponse, user_service_server::UserService,
};
use tonic::{Request, Response, Status};

use crate::domain::ports::UserPorts;

pub struct UserHandler<C: UserPorts> {
    pub(crate) ctx: C,
}

impl<C: UserPorts> UserHandler<C> {
    pub fn new(ctx: C) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl<C: UserPorts + Send + Sync + 'static> UserService for UserHandler<C> {
    async fn health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        rpc::create_user::execute(&self.ctx, request).await
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        rpc::get_user::execute(&self.ctx, request).await
    }

    async fn get_user_by_handle(
        &self,
        request: Request<GetUserByHandleRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        rpc::get_user_by_handle::execute(&self.ctx, request).await
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        rpc::list_users::execute(&self.ctx, request).await
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        rpc::update_user::execute(&self.ctx, request).await
    }

    async fn deactivate_user(
        &self,
        request: Request<DeactivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        rpc::deactivate_user::execute(&self.ctx, request).await
    }

    async fn activate_user(
        &self,
        request: Request<ActivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        rpc::activate_user::execute(&self.ctx, request).await
    }

    async fn change_role(
        &self,
        request: Request<ChangeRoleRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        rpc::change_role::execute(&self.ctx, request).await
    }
}
