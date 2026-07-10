//! Test-mode database access for Pythia.
//!
//! Wraps the SQLite database that stores test profiles and their scheduled CAN
//! messages, exposing the diesel models and a small connection/query API so
//! callers never have to touch diesel directly.

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
