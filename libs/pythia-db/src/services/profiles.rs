//! Services for `test_profile` rows.

use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::Result;

/// Return the names of every test profile, ordered alphabetically.
pub fn get_all_names(conn: &mut SqliteConnection) -> Result<Vec<String>> {
    use crate::schema::test_profile::dsl::{name, test_profile};

    Ok(test_profile.select(name).order(name.asc()).load::<String>(conn)?)
}
