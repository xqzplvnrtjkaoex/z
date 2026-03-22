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
        let msg = err.to_string();
        // PostgreSQL unique violation error code: 23505
        if msg.contains("duplicate key")
            || msg.contains("unique constraint")
            || msg.contains("23505")
        {
            RepositoryError::UniqueViolation("handle".to_string())
        } else {
            RepositoryError::Database(msg)
        }
    }
}
