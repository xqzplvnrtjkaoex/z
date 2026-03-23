use super::repository_error::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("user not found")]
    UserNotFound,

    #[error("handle already taken")]
    HandleTaken,

    #[error("handle is reserved")]
    HandleReserved,

    #[error("invalid handle: {0}")]
    InvalidHandle(String),

    #[error("invalid name: {0}")]
    InvalidName(String),

    #[error("insufficient role: {reason}")]
    InsufficientRole { reason: String },

    #[error("self-modification is not allowed")]
    SelfModification,

    #[error("user is inactive")]
    UserInactive,

    #[error("user is already active")]
    UserAlreadyActive,

    #[error("user is already inactive")]
    UserAlreadyInactive,

    #[error("owner role cannot be assigned via API")]
    OwnerRoleRejected,

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<RepositoryError> for UserError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => UserError::UserNotFound,
            RepositoryError::UniqueViolation(field) => {
                if field.contains("handle") {
                    UserError::HandleTaken
                } else {
                    UserError::Internal(format!("unique violation: {field}"))
                }
            }
            RepositoryError::Database(msg) => UserError::Internal(msg),
        }
    }
}

impl From<UserError> for tonic::Status {
    fn from(err: UserError) -> Self {
        match err {
            UserError::UserNotFound => tonic::Status::not_found(err.to_string()),
            UserError::HandleTaken => tonic::Status::already_exists(err.to_string()),
            UserError::HandleReserved => tonic::Status::invalid_argument(err.to_string()),
            UserError::InvalidHandle(_) => tonic::Status::invalid_argument(err.to_string()),
            UserError::InvalidName(_) => tonic::Status::invalid_argument(err.to_string()),
            UserError::InsufficientRole { .. } => tonic::Status::permission_denied(err.to_string()),
            UserError::SelfModification => tonic::Status::permission_denied(err.to_string()),
            UserError::UserInactive => tonic::Status::failed_precondition(err.to_string()),
            UserError::UserAlreadyActive => tonic::Status::failed_precondition(err.to_string()),
            UserError::UserAlreadyInactive => tonic::Status::failed_precondition(err.to_string()),
            UserError::OwnerRoleRejected => tonic::Status::invalid_argument(err.to_string()),
            UserError::Internal(_) => tonic::Status::internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn should_map_user_not_found_to_not_found_status() {
        let status: tonic::Status = UserError::UserNotFound.into();
        assert_eq!(status.code(), Code::NotFound);
    }

    #[test]
    fn should_map_handle_taken_to_already_exists_status() {
        let status: tonic::Status = UserError::HandleTaken.into();
        assert_eq!(status.code(), Code::AlreadyExists);
    }

    #[test]
    fn should_map_handle_reserved_to_invalid_argument_status() {
        let status: tonic::Status = UserError::HandleReserved.into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn should_map_invalid_handle_to_invalid_argument_status() {
        let status: tonic::Status = UserError::InvalidHandle("too short".to_string()).into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn should_map_invalid_name_to_invalid_argument_status() {
        let status: tonic::Status = UserError::InvalidName("too long".to_string()).into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn should_map_insufficient_role_to_permission_denied_status() {
        let status: tonic::Status = UserError::InsufficientRole {
            reason: "admin required".to_string(),
        }
        .into();
        assert_eq!(status.code(), Code::PermissionDenied);
    }

    #[test]
    fn should_map_self_modification_to_permission_denied_status() {
        let status: tonic::Status = UserError::SelfModification.into();
        assert_eq!(status.code(), Code::PermissionDenied);
    }

    #[test]
    fn should_map_user_inactive_to_failed_precondition_status() {
        let status: tonic::Status = UserError::UserInactive.into();
        assert_eq!(status.code(), Code::FailedPrecondition);
    }

    #[test]
    fn should_map_user_already_active_to_failed_precondition_status() {
        let status: tonic::Status = UserError::UserAlreadyActive.into();
        assert_eq!(status.code(), Code::FailedPrecondition);
    }

    #[test]
    fn should_map_user_already_inactive_to_failed_precondition_status() {
        let status: tonic::Status = UserError::UserAlreadyInactive.into();
        assert_eq!(status.code(), Code::FailedPrecondition);
    }

    #[test]
    fn should_map_owner_role_rejected_to_invalid_argument_status() {
        let status: tonic::Status = UserError::OwnerRoleRejected.into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn should_map_internal_to_internal_status() {
        let status: tonic::Status = UserError::Internal("db error".to_string()).into();
        assert_eq!(status.code(), Code::Internal);
    }
}
