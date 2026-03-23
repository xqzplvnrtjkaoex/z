use axum::{extract::Request, middleware::Next, response::Response};

use crate::error::AppError;

/// Validated caller identity extracted from HTTP request headers.
#[derive(Clone, Debug)]
pub struct CallerContext {
    pub caller_id: String,
    pub caller_role: String,
}

impl CallerContext {
    /// Inject caller context into tonic gRPC request metadata.
    pub fn inject_into<T>(&self, request: &mut tonic::Request<T>) {
        use madome_common::headers;
        let metadata = request.metadata_mut();
        if let Ok(val) = self.caller_id.parse() {
            metadata.insert(headers::X_CALLER_ID, val);
        }
        if let Ok(val) = self.caller_role.parse() {
            metadata.insert(headers::X_CALLER_ROLE, val);
        }
    }
}

/// Axum middleware that extracts caller identity headers and stores them
/// in request extensions as `CallerContext`.
///
/// The middleware is permissive: it inserts `CallerContext` into extensions
/// only when both headers are present and valid. Handlers that require
/// authentication context use `Extension<CallerContext>` and will return
/// an error if it is missing.
pub async fn extract_caller_context(
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
        .map(|s| s.to_string());

    let caller_role = request
        .headers()
        .get(headers::X_CALLER_ROLE)
        .map(|v| {
            v.to_str()
                .map_err(|_| AppError::BadRequest("invalid x-caller-role header".to_string()))
        })
        .transpose()?
        .map(|s| s.to_string());

    if let (Some(id), Some(role)) = (caller_id, caller_role) {
        request.extensions_mut().insert(CallerContext {
            caller_id: id,
            caller_role: role,
        });
    }

    Ok(next.run(request).await)
}
