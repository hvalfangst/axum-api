use axum::{Router, ServiceExt, routing::post};
use diesel::{PgConnection, r2d2::{ConnectionManager, Pool}};
use crate::db::create_shared_connection_pool;
use crate::routes::locations::router::locations_route;
use crate::routes::users;
use crate::util::load_database_url;
use crate::routes::users::router::users_route;
use crate::users::router::create_user_handler;

mod services;
mod model;
mod routes;
mod util;
mod db;
mod schema;

pub struct ConnectionPool {
    pool: Pool<ConnectionManager<PgConnection>>,
}

#[tokio::main]
async fn main() {
    let database_url = load_database_url();
    let shared_connection_pool = create_shared_connection_pool(database_url);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(users_route(shared_connection_pool.clone())
            .nest("/",
                  locations_route(shared_connection_pool.clone()))

                .into_make_service())
        .await
        .unwrap();
}





