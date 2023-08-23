pub mod router {
    use std::sync::Arc;
    use std::time::Instant;
    use serde_json::{json, Value};
    use axum::{Router, http::StatusCode, Json, response::IntoResponse, extract::State, routing::post, routing::MethodRouter, extract};
    use crate::{
        ConnectionPool,
        model::{UpsertLocation, Location},
        services::locations::service::DbExecutor,
    };

    // - - - - - - - - - - - [ROUTE] - - - - - - - - - - -

    pub fn locations_route(shared_connection_pool: Arc<ConnectionPool>) -> Router {
        Router::new()
            .route("/locations", axum::routing::post(create_location_handler))
            .route("/locations/:location_id", axum::routing::get(read_location_handler))
            .route("/locations/:location_id", axum::routing::put(update_location_handler))
            .route("/locations/:location_id", axum::routing::delete(delete_location_handler))
            .with_state(shared_connection_pool)
    }


    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn create_location_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        Json(upsert_location): Json<UpsertLocation>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut locations = DbExecutor::new(connection);

        match locations.create(upsert_location) {
            Ok(new_location) => Ok((StatusCode::CREATED, Json(new_location))),
            Err(err) => {
                eprintln!("Error creating location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create location"}))))
            }
        }
    }

    pub async fn read_location_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut locations = DbExecutor::new(connection);

        match locations.read(location_id) {
            Ok(location) => {
                if let Some(location) = location {
                    Ok((StatusCode::OK, Json(location)))
                } else {
                    Err((StatusCode::NOT_FOUND, Json(json!({"error": "Location not found"}))))
                }
            },
            Err(err) => {
                eprintln!("Error reading location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read location"}))))
            }
        }
    }

    pub async fn update_location_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        path: extract::Path<(i32,)>,
        Json(upsert_location): Json<UpsertLocation>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut locations = DbExecutor::new(connection);

        match locations.update(location_id, upsert_location) {
            Ok(updated_location) => Ok((StatusCode::OK, Json(updated_location))),
            Err(err) => {
                eprintln!("Error updating location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update location"}))))
            }
        }
    }

    pub async fn delete_location_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut locations = DbExecutor::new(connection);

        match locations.delete(location_id) {
            Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
            Err(err) => {
                eprintln!("Error deleting location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to delete location"}))))
            }
        }
    }


}
