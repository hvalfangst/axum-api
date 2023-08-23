// @generated automatically by Diesel CLI.

diesel::table! {
    users (user_id) {
        user_id -> Int4,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 100]
        password -> Varchar,
        #[max_length = 100]
        fullname -> Varchar,
        role_id -> Int4,
    }
}

diesel::table! {
    locations (location_id) {
        location_id -> Int4,
        #[max_length = 100]
        star_system -> Varchar,
        #[max_length = 100]
        area -> Varchar,
    }
}

// - - - - - - - - - - - [TODO] - - - - - - - - - - -

// diesel::table! {
//     players (player_id) {
//         player_id -> Int4,
//         user_id -> Int4,
//         active_ship_id -> Int4,
//         location_id -> Int4,
//     }
// }
//
// diesel::table! {
//     roles (role_id) {
//         role_id -> Int4,
//         #[max_length = 50]
//         role_name -> Varchar,
//     }
// }
//
// diesel::table! {
//     ships (ship_id) {
//         ship_id -> Int4,
//         #[max_length = 100]
//         name -> Varchar,
//         // #[sql_name = "type", max_length = 50]
//         // type_ -> Nullable<Varchar>,
//         description -> Nullable<Text>,
//         empire_id -> Int4,
//     }
// }

// diesel::table! {
//     empires (empire_id) {
//         empire_id -> Int4,
//         #[max_length = 100]
//         name -> Varchar,
//         #[max_length = 100]
//         slogan -> Nullable<Varchar>,
//         location_id -> Int4,
//         description -> Nullable<Text>,
//     }
// }
//


// diesel::joinable!(empires -> locations (location_id));
// diesel::joinable!(players -> locations (location_id));
// diesel::joinable!(players -> ships (active_ship_id));
// diesel::joinable!(players -> users (user_id));
// diesel::joinable!(ships -> empires (empire_id));
// diesel::joinable!(users -> roles (role_id));
//
// diesel::allow_tables_to_appear_in_same_query!(
//     empires,
//     locations,
//     players,
//     roles,
//     ships,
//     users,
// );
