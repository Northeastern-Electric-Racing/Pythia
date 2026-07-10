use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use pythia_db::TestCanMessageEntry;
use serde::Deserialize;

use crate::DbPool;

/// Query parameters for [`list_profile_messages`].
#[derive(Deserialize)]
pub struct ProfileQuery {
    /// Name of the test profile whose messages to fetch.
    profile: String,
}

/// `GET /messages?profile=<name>` — all CAN messages for the given profile,
/// ordered by ascending offset. Returns 404 if the profile doesn't exist.
pub async fn list_profile_messages(
    State(pool): State<DbPool>,
    Query(query): Query<ProfileQuery>,
) -> Result<Json<Vec<TestCanMessageEntry>>, StatusCode> {
    let messages = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match pythia_db::services::messages::get_by_profile_name(&mut conn, &query.profile) {
            Ok(messages) => Ok(messages),
            Err(pythia_db::Error::ProfileNotFound(_)) => Err(StatusCode::NOT_FOUND),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    Ok(Json(messages))
}
