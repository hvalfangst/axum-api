use std::sync::Arc;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::ConnectionPool;

pub fn create_shared_connection_pool(database_url: String, max_size: u32) -> Arc<ConnectionPool> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Failed to create connection pool");

    Arc::new(ConnectionPool {
        pool: pool.clone()
    })
}