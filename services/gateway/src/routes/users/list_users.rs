use axum::{
    Json,
    extract::{Extension, State},
    response::{AppendHeaders, IntoResponse},
};
use itertools::Itertools;
use madome_common::{caller::CallerIdentity, headers::X_NEXT_CURSOR};
use madome_proto::user::ListUsersRequest;
use serde_qs::axum::QsQuery;

use crate::{error::AppError, model::User, payload::user::ListUsersQuery, state::AppState};

pub async fn list_users(
    State(state): State<AppState>,
    Extension(identity): Extension<CallerIdentity>,
    QsQuery(mut query): QsQuery<ListUsersQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mut request = tonic::Request::new(ListUsersRequest {
        limit: query.limit(),
        cursor: query.cursor(),
        include_inactive: query.include_inactive(),
    });
    identity.inject_into(&mut request);

    let response = state.user_client.clone().list_users(request).await?;

    let inner = response.into_inner();
    let headers = inner
        .next_cursor
        .map(|c| AppendHeaders([(X_NEXT_CURSOR, c)]));
    let users: Vec<User> = inner.users.into_iter().map_into().collect();

    Ok((headers, Json(users)))
}
