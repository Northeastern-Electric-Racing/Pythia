use axum::{Router, routing::get};
use diesel::Connection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use tracing::{info, warn};

/// Shared r2d2 connection pool over the SQLite test-mode database.
/// Cloneable (it's `Arc`-backed) so it can be handed to axum as router state.
type DbPool = Pool<ConnectionManager<SqliteConnection>>;

mod controllers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    info!("Initializing Pythia DB");

    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be specified");

    info!("Beginning DB migration w/ temporary connection...");

    // Run migrations on a temporary standalone connection
    let migrate_url = db_url.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = SqliteConnection::establish(&migrate_url)
            .expect("failed to open migration connection");
        match pythia_db::run_migrations(&mut conn) {
            Ok(()) => info!("Successfully migrated DB!"),
            Err(e) => warn!("Encountered Error: {}", e),
        }
    })
    .await
    .expect("migration task panicked");

    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool: DbPool = Pool::builder()
        .build(manager)
        .expect("failed to build database connection pool");

    // Axum router
    let app = Router::new()
        .route("/profiles", get(controllers::profiles::list_profile_names))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind listener");
    println!("Pythia listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("server error");
}
