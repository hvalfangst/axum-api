pub mod router {
    use std::sync::Arc;
    use std::time::Instant;
    use serde_json::{json, Value};
    use axum::{Router, http::StatusCode, Json, response::IntoResponse, extract::State, routing::post, routing::MethodRouter, extract};
    use crate::{
        ConnectionPool,
        model::{UpsertUser, User},
        services::users::service::DbExecutor,
    };

    // - - - - - - - - - - - [ROUTE] - - - - - - - - - - -

    pub fn users_route(shared_connection_pool: Arc<ConnectionPool>) -> Router {
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
                if user.is_empty() {
                    return Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))));
                }
                Ok((StatusCode::OK, Json(user)))
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
