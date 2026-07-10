//! Query/read operations over the test-mode database.
//!
//! Each service takes a mutable diesel connection and returns domain types,
//! leaving connection management (pooling, env loading) to the caller. This is
//! the layer both the Pythia app's controllers and other consumers (e.g.
//! Calypso) build on top of.

pub mod messages;
pub mod profiles;
