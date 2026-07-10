use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use pythia_db::{NewCanMessageInput, TestCanMessageEntry};
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

/// `POST /messages?profile=<name>` — add a CAN message to the named profile.
///
/// The target profile is identified by the `profile` query parameter; the
/// message fields come from the JSON body. Returns the created message with
/// `201 Created`, `404 Not Found` if the profile doesn't exist, or
/// `400 Bad Request` if the message violates a database constraint.
pub async fn create_profile_message(
    State(pool): State<DbPool>,
    Query(query): Query<ProfileQuery>,
    Json(body): Json<NewCanMessageInput>,
) -> Result<(StatusCode, Json<TestCanMessageEntry>), StatusCode> {
    let message = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match pythia_db::services::messages::create(&mut conn, &query.profile, body) {
            Ok(message) => Ok(message),
            Err(pythia_db::Error::ProfileNotFound(_)) => Err(StatusCode::NOT_FOUND),
            Err(pythia_db::Error::InvalidCanMessage(_)) => Err(StatusCode::BAD_REQUEST),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    Ok((StatusCode::CREATED, Json(message)))
}
