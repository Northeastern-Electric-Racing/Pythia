//! Axum HTTP handlers. Each controller validates request inputs, calls a
//! `pythia_db` service to do the actual DB work, and maps the result to an
//! HTTP response.

pub mod profiles;
