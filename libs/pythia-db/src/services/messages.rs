//! Services for `can_message` rows.

use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::services::profiles;
use crate::{Result, TestCanMessageEntry};

/// Load every CAN message belonging to the named profile, ordered by ascending
/// offset.
///
/// Returns [`Error::ProfileNotFound`](crate::Error::ProfileNotFound) if no
/// profile with that name exists.
pub fn get_by_profile_name(
    conn: &mut SqliteConnection,
    profile_name: &str,
) -> Result<Vec<TestCanMessageEntry>> {
    use crate::schema::can_message::dsl::{can_message, offset_ms, profile_id};

    let profile = profiles::get_by_name(conn, profile_name)?;

    Ok(can_message
        .filter(profile_id.eq(profile.id))
        .select(TestCanMessageEntry::as_select())
        .order(offset_ms.asc())
        .load(conn)?)
}
