use axum::{Json, extract::State, http::StatusCode};

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
