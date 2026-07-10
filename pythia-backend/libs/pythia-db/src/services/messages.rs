//! Services for `can_message` rows.

use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use crate::services::profiles;
use crate::{Error, NewCanMessage, NewCanMessageInput, Result, TestCanMessageEntry};

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

/// Add a CAN message to the named profile, returning the inserted row.
///
/// Returns [`Error::ProfileNotFound`] if no profile with that name exists, and
/// [`Error::InvalidCanMessage`] if the message violates a database constraint
/// (out-of-range `can_id`, unknown `mode`, oversized `data`, or a period/mode
/// mismatch).
pub fn create(
    conn: &mut SqliteConnection,
    profile_name: &str,
    input: NewCanMessageInput,
) -> Result<TestCanMessageEntry> {
    use crate::schema::can_message::dsl::can_message;

    let profile = profiles::get_by_name(conn, profile_name)?;

    let new_message = NewCanMessage {
        profile_id: profile.id,
        can_id: input.can_id,
        is_extended: input.is_extended,
        data: input.data,
        mode: input.mode,
        offset_ms: input.offset_ms,
        period_ms: input.period_ms,
    };

    diesel::insert_into(can_message)
        .values(new_message)
        .returning(TestCanMessageEntry::as_returning())
        .get_result(conn)
        .map_err(|e| match e {
            DieselError::DatabaseError(DatabaseErrorKind::CheckViolation, info) => {
                Error::InvalidCanMessage(info.message().to_owned())
            }
            other => Error::Query(other),
        })
}
