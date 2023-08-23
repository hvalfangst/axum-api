use diesel::prelude::*;
use serde_derive::{Serialize, Deserialize};
use crate::schema::{users, locations};

// - - - - - - - - - - - [USERS] - - - - - - - - - - -

#[derive(Serialize, Debug, Clone, Queryable)]
#[diesel(table_name = users)]
pub struct User {
    pub user_id: i32,
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub role_id: i32,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct UpsertUser {
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub role_id: i32,
}

// - - - - - - - - - - - [LOCATIONS] - - - - - - - - - - -

#[derive(Serialize, Debug, Clone, Queryable)]
#[diesel(table_name = locations)]
pub struct Location {
    pub location_id: i32,
    pub star_system: String,
    pub area: String,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = locations)]
pub struct UpsertLocation {
    pub star_system: String,
    pub area: String,
}

// - - - - - - - - - - - [TODO] - - - - - - - - - - -

//
// #[derive(Serialize, Debug, Clone, Queryable)]
// struct Empire {
//     pub empire_id: i32,
//     pub name: String,
//     pub slogan: Option<String>,
//     pub location_id: i32,
//     pub description: Option<String>,
// }
//
// #[derive(Serialize, Debug, Clone, Queryable)]
// struct Ship {
//     pub ship_id: i32,
//     pub name: String,
//     pub ship_type: Option<String>,
//     pub description: Option<String>,
//     pub empire_id: i32,
// }
//
// #[derive(Serialize, Debug, Clone, Queryable)]
// struct Player {
//     pub player_id: i32,
//     pub user_id: i32,
//     pub active_ship_id: i32,
//     pub location_id: i32,
// }
//
// #[derive(Serialize, Debug, Clone, Queryable)]
// struct Role {
//     pub role_id: i32,
//     pub role_name: String,
// }