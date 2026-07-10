//! Test-mode database access for Pythia.
//!
//! Wraps the SQLite database that stores test profiles and their scheduled CAN
//! messages, exposing the diesel models and a small connection/query API so
//! callers never have to touch diesel directly.

use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub mod models;
mod schema;
pub mod services;

/// Migrations embedded at compile time from this crate's `migrations/` directory.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub use models::{TestCanMessageEntry, TestProfile};

/// Errors that can occur while connecting to or querying the test database.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The `DATABASE_URL` environment variable was not set.
    #[error("DATABASE_URL environment variable is not set: {0}")]
    MissingDatabaseUrl(#[from] std::env::VarError),

    /// Establishing the SQLite connection failed.
    #[error("could not connect to test database at {url}")]
    Connection {
        url: String,
        #[source]
        source: diesel::ConnectionError,
    },

    /// No profile with the requested name exists.
    #[error("test profile '{0}' not found")]
    ProfileNotFound(String),

    /// A query against the database failed.
    #[error(transparent)]
    Query(#[from] diesel::result::Error),

    /// Running database migrations failed.
    #[error("failed to run migrations: {0}")]
    Migration(String),
}

/// Result alias for this crate's fallible operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Run all pending migrations against the given connection.
///
/// Safe to call on an already-migrated database: diesel skips migrations that
/// are already recorded in `__diesel_schema_migrations`.
pub fn run_migrations(conn: &mut SqliteConnection) -> Result<()> {
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| Error::Migration(e.to_string()))?;
    Ok(())
}

/// A connection to the test-mode SQLite database.
pub struct TestModeDb {
    conn: SqliteConnection,
}

impl TestModeDb {
    /// Look up a test profile by its unique name.
    pub fn find_profile_by_name(&mut self, profile_name: &str) -> Result<Option<TestProfile>> {
        use schema::test_profile::dsl::{name, test_profile};

        Ok(test_profile
            .filter(name.eq(profile_name))
            .select(TestProfile::as_select())
            .first(&mut self.conn)
            .optional()?)
    }

    /// Load every CAN message belonging to the named profile, ordered by
    /// ascending offset.
    ///
    /// Returns [`Error::ProfileNotFound`] if no profile with that name exists.
    pub fn load_profile_messages(
        &mut self,
        profile_name: &str,
    ) -> Result<Vec<TestCanMessageEntry>> {
        use schema::can_message::dsl::{can_message, offset_ms, profile_id};

        let profile = self
            .find_profile_by_name(profile_name)?
            .ok_or_else(|| Error::ProfileNotFound(profile_name.to_owned()))?;

        Ok(can_message
            .filter(profile_id.eq(profile.id))
            .select(TestCanMessageEntry::as_select())
            .order(offset_ms.asc())
            .load(&mut self.conn)?)
    }
}
