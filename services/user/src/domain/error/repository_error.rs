#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("entity not found")]
    NotFound,

    #[error("unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("database error: {0}")]
    Database(String),
}
