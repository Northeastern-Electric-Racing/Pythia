//! Services for `test_profile` rows.

use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use crate::{Error, NewTestProfile, Result, TestProfile};

/// Return the names of every test profile, ordered alphabetically.
pub fn get_all_names(conn: &mut SqliteConnection) -> Result<Vec<String>> {
    use crate::schema::test_profile::dsl::{name, test_profile};

    Ok(test_profile.select(name).order(name.asc()).load::<String>(conn)?)
}

/// Look up a test profile by its unique name, returning `None` if none exists.
pub fn find_by_name(conn: &mut SqliteConnection, profile_name: &str) -> Result<Option<TestProfile>> {
    use crate::schema::test_profile::dsl::{name, test_profile};

    Ok(test_profile
        .filter(name.eq(profile_name))
        .select(TestProfile::as_select())
        .first(conn)
        .optional()?)
}

/// Look up a test profile by name, erroring with [`Error::ProfileNotFound`]
/// when no profile with that name exists.
pub fn get_by_name(conn: &mut SqliteConnection, profile_name: &str) -> Result<TestProfile> {
    find_by_name(conn, profile_name)?
        .ok_or_else(|| Error::ProfileNotFound(profile_name.to_owned()))
}

/// Create a new test profile with the given name, returning the inserted row.
///
/// Returns [`Error::ProfileAlreadyExists`] if a profile with that name already
/// exists (the `name` column is `UNIQUE`).
pub fn create(conn: &mut SqliteConnection, profile_name: &str) -> Result<TestProfile> {
    use crate::schema::test_profile::dsl::test_profile;

    diesel::insert_into(test_profile)
        .values(NewTestProfile { name: profile_name })
        .returning(TestProfile::as_returning())
        .get_result(conn)
        .map_err(|e| match e {
            DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                Error::ProfileAlreadyExists(profile_name.to_owned())
            }
            other => Error::Query(other),
        })
}
