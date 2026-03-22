use crate::domain::ports::user_repository::UserRepository;
use crate::domain::ports::{UserConfig, UserPorts};

use super::postgres::user_repository::PostgresUserRepository;

pub struct UserContext {
    user_repo: PostgresUserRepository,
}

impl UserContext {
    pub fn new(user_repo: PostgresUserRepository) -> Self {
        Self { user_repo }
    }
}

impl UserPorts for UserContext {
    fn user_repo(&self) -> &impl UserRepository {
        &self.user_repo
    }
}

impl UserConfig for UserContext {}
