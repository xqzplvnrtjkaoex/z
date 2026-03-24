#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("entity not found")]
    NotFound,

    #[error("unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("database error: {0}")]
    Database(String),
}

impl From<sea_orm::DbErr> for RepositoryError {
    fn from(err: sea_orm::DbErr) -> Self {
        match err.sql_err() {
            Some(sea_orm::SqlErr::UniqueConstraintViolation(msg)) => {
                RepositoryError::UniqueViolation(msg)
            }
            _ => RepositoryError::Database(err.to_string()),
        }
    }
}
