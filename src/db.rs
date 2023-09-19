use std::sync::Arc;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

#[derive(Clone)]
pub struct ConnectionPool {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

pub fn create_shared_connection_pool(database_url: String, max_size: u32) -> ConnectionPool {
    eprintln!("Attempting to create shared connection pool");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    eprintln!("ConnectionManager created");

    let pool = Pool::builder()
        .max_size(max_size)
        .build(manager)
        .unwrap();

    eprintln!("Pool created");

    ConnectionPool {
        pool,
    }
}