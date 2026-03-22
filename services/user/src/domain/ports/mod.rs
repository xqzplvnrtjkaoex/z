pub mod user_repository;

use user_repository::UserRepository;

pub trait UserPorts {
    fn user_repo(&self) -> &impl UserRepository;
}

pub trait UserConfig {}
