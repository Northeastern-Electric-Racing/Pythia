use axum::{Json, extract::State, http::StatusCode};
use pythia_db::TestProfile;
use serde::Deserialize;

use crate::DbPool;

/// `GET /profiles` — list the names of all test profiles.
///
/// Checks a connection out of the pool and runs the blocking diesel query on a
/// blocking thread so the async runtime isn't stalled.
pub async fn list_profile_names(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let names = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        pythia_db::services::profiles::get_all_names(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    Ok(Json(names))
}

/// Request body for [`create_profile`].
#[derive(Deserialize)]
pub struct NewProfileBody {
    /// Name for the new test profile; must be unique.
    name: String,
}

/// `POST /profiles` — create a new test profile from a JSON body
/// `{ "name": "<name>" }`. Returns the created profile with `201 Created`, or
/// `409 Conflict` if a profile with that name already exists.
pub async fn create_profile(
    State(pool): State<DbPool>,
    Json(body): Json<NewProfileBody>,
) -> Result<(StatusCode, Json<TestProfile>), StatusCode> {
    let profile = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match pythia_db::services::profiles::create(&mut conn, &body.name) {
            Ok(profile) => Ok(profile),
            Err(pythia_db::Error::ProfileAlreadyExists(_)) => Err(StatusCode::CONFLICT),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    Ok((StatusCode::CREATED, Json(profile)))
}
