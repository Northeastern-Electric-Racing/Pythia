//! Services for `test_profile` rows.

use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::{Error, Result, TestProfile};

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
