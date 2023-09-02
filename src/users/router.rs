pub mod router {
    use std::{
        sync::Arc,
        time::Instant,
    };
    use serde_json::{json, Value};
    use axum::{extract, extract::State, http::StatusCode, Json, response::IntoResponse, Router};
    use crate::{
        ConnectionPool,
        users::{
            service::service::DbExecutor,
            model::UpsertUser
        },
    };

    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn users_routes(shared_connection_pool: Arc<ConnectionPool>) -> Router {
        Router::new()
            .route("/users", axum::routing::post(create_user_handler))
            .route("/users/:user_id", axum::routing::get(read_user_handler))
            .route("/users/:user_id", axum::routing::put(update_user_handler))
            .route("/users/:user_id", axum::routing::delete(delete_user_handler))
            .with_state(shared_connection_pool)
    }


    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn create_user_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        Json(body): Json<UpsertUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = DbExecutor::new(connection);

        let create_user_begin = Instant::now();
        let user = users.create(body).expect("Create user failed");
        let create_user_end = create_user_begin.elapsed();
        println!("Time to create user: {:?}", create_user_end);

        Ok((StatusCode::CREATED, Json(user)))
    }

    pub async fn read_user_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (user_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = DbExecutor::new(connection);

        match users.read(user_id) {
            Ok(user) => {
                if let Some(user) = user {
                    Ok((StatusCode::OK, Json(user)))
                } else {
                    Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))))
                }
            },
            Err(err) => {
                eprintln!("Error reading user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read user"}))))
            }
        }
    }

    pub async fn update_user_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        path: extract::Path<(i32,)>,
        Json(update_user): Json<UpsertUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (user_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = DbExecutor::new(connection);

        match users.update(user_id, update_user) {
            Ok(updated_user) => Ok((StatusCode::OK, Json(updated_user))),
            Err(diesel::result::Error::NotFound) => {
                Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))))
            },
            Err(err) => {
                eprintln!("Error updating user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update user"}))))
            }
        }
    }

    pub async fn delete_user_handler(
        State(shared_state): State<Arc<ConnectionPool>>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (user_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = DbExecutor::new(connection);

        match users.delete(user_id) {
            Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
            Err(err) => {
                eprintln!("Error deleting user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to delete user"}))))
            }
        }
    }

}
