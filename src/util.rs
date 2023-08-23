use std::env;
use dotenvy::dotenv;

pub fn load_database_url() -> String {
    dotenv().ok();
    env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
}