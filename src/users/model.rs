use diesel::prelude::*;
use serde_derive::{Serialize, Deserialize};
use crate::schema::users;

#[derive(Serialize, Debug, Clone, Queryable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub role_id: i32,
}

#[derive(Debug, Clone, Insertable, Deserialize, Serialize)]
#[diesel(table_name = users)]
pub struct UpsertUser {
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub role_id: i32,
}