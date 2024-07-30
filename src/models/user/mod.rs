use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub hashed_password: String,
    pub balance: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Debug, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub hashed_password: String,
    pub balance: i32,
}
