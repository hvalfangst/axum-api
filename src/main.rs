use crate:: {
    common::db::create_shared_connection_pool,
    locations::router::router::locations_route,
    empires::router::router::empires_route,
    users::router::router::users_route,
    common::util::load_environment_variable,
};

mod locations;mod users;mod schema;mod common;
mod empires;

#[tokio::main]
async fn main() {
    let database_url = load_environment_variable("DEV_DB");
    let shared_connection_pool = create_shared_connection_pool(database_url, 1);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(users_route(shared_connection_pool.clone())
            .nest("/", locations_route(shared_connection_pool.clone()))
            .nest("/", empires_route(shared_connection_pool.clone()))
                .into_make_service())
        .await
        .unwrap();
}





