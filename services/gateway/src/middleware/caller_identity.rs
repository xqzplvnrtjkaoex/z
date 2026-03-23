use axum::{extract::Request, middleware::Next, response::Response};
use madome_common::caller::{CallerIdentity, CallerRole};

use crate::error::AppError;

/// Axum middleware that extracts caller identity headers and stores them
/// in request extensions as [`CallerIdentity`].
///
/// The middleware is permissive: it inserts `CallerIdentity` into extensions
/// only when both headers are present and valid. Handlers that require
/// authentication context use `Extension<CallerIdentity>` and will return
/// an error if it is missing.
pub async fn extract_caller_identity(
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    use madome_common::headers;

    let caller_id = request
        .headers()
        .get(headers::X_CALLER_ID)
        .map(|v| {
            v.to_str()
                .map_err(|_| AppError::BadRequest("invalid x-caller-id header".to_string()))
        })
        .transpose()?
        .map(|s| {
            s.parse::<uuid::Uuid>()
                .map_err(|_| AppError::BadRequest("invalid x-caller-id format".to_string()))
        })
        .transpose()?;

    let caller_role = request
        .headers()
        .get(headers::X_CALLER_ROLE)
        .map(|v| {
            v.to_str()
                .map_err(|_| AppError::BadRequest("invalid x-caller-role header".to_string()))
        })
        .transpose()?
        .map(|s| {
            s.parse::<CallerRole>()
                .map_err(|_| AppError::BadRequest("invalid x-caller-role value".to_string()))
        })
        .transpose()?;

    if let (Some(id), Some(role)) = (caller_id, caller_role) {
        request.extensions_mut().insert(CallerIdentity {
            caller_id: id,
            caller_role: role,
        });
    }

    Ok(next.run(request).await)
}
