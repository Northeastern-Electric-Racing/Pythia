use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::test_profile)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TestProfile {
    pub id: i32,
    pub name: String,
}

/// Column values for inserting a new [`TestProfile`]. The `id` is assigned by
/// the database.
#[derive(Insertable)]
#[diesel(table_name = crate::schema::test_profile)]
pub struct NewTestProfile<'a> {
    pub name: &'a str,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::can_message)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TestCanMessageEntry {
    pub id: i32,
    pub profile_id: i32,
    pub can_id: i32,
    pub is_extended: i32,
    pub data: Vec<u8>,
    pub mode: String,
    pub offset_ms: i32,
    pub period_ms: Option<i32>,
}

/// Fields a caller supplies to add a CAN message to a profile. Excludes `id`
/// (assigned by the database) and `profile_id` (resolved from the target
/// profile's name), so it can be deserialized directly from a request body.
#[derive(Deserialize)]
pub struct NewCanMessageInput {
    pub can_id: i32,
    pub is_extended: i32,
    pub data: Vec<u8>,
    pub mode: String,
    pub offset_ms: i32,
    pub period_ms: Option<i32>,
}

/// Column values for inserting a new CAN message row, including the resolved
/// `profile_id`.
#[derive(Insertable)]
#[diesel(table_name = crate::schema::can_message)]
pub struct NewCanMessage {
    pub profile_id: i32,
    pub can_id: i32,
    pub is_extended: i32,
    pub data: Vec<u8>,
    pub mode: String,
    pub offset_ms: i32,
    pub period_ms: Option<i32>,
}

