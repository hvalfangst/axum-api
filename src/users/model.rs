use diesel::prelude::*;
use regex::Regex;
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

#[derive(Serialize, Debug, Clone, Queryable)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: i32,
    pub role_name: String
}

#[derive(Debug, Clone, Insertable, Deserialize, Serialize)]
#[diesel(table_name = users)]
pub struct UpsertUser {
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub role_id: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub role: String
}

impl UpsertUser {
    pub fn is_valid_email(&self) -> bool {
        let email_pattern = Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").unwrap();
        email_pattern.is_match(&self.email)
    }
}