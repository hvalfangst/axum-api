pub mod router {
    use serde_json::{json, Value};
    use axum::{
        Router, http::StatusCode, Json, response::IntoResponse, extract::State, extract,
    };
    use http::HeaderMap;
    use crate::{
        common::db::ConnectionPool,
        empires::{
            service::service::EmpiresTable as empiresTable,
            model::UpsertEmpire
        },
        users::model::UserRole,
        common::security::{enforce_role_policy, decode_claims}
    };

    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn empires_route(shared_connection_pool: ConnectionPool) -> Router {
        Router::new()
            .route("/empires", axum::routing::post(create_empire_handler))
            .route("/empires/:empire_id", axum::routing::get(read_empire_handler))
            .route("/empires/:empire_id", axum::routing::put(update_empire_handler))
            .route("/empires/:empire_id", axum::routing::delete(delete_empire_handler))
            .with_state(shared_connection_pool)
    }

    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn create_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        Json(upsert_empire): Json<UpsertEmpire>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

        // Decode claims from bearer token header
        let claims = match decode_claims(&headers) {
            Ok(claims) => claims,
            Err((status_code, json_value)) => return Err((status_code, json_value)),
        };

        // Ensure that the user derived from claims exists and has the role 'WRITER' or higher
        let authorization = enforce_role_policy(&shared_state, &claims, UserRole::WRITER).await;

        match authorization {
            Ok(_authorized_user) => {
                let connection = shared_state.pool.get()
                    .expect("Failed to acquire connection from pool");

                match empiresTable::new(connection).create(upsert_empire) {
                    Ok(new_empire) => Ok((StatusCode::CREATED, Json(new_empire))),
                    Err(err) => {
                        eprintln!("Error creating empire: {:?}", err);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create empire"}))))
                    }
                }
            }
            Err(err) => Err(err)
        }
    }


    pub async fn read_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;

        // Decode claims from bearer token header
        let claims = match decode_claims(&headers) {
            Ok(claims) => claims,
            Err((status_code, json_value)) => return Err((status_code, json_value)),
        };

        // Ensure that the user derived from claims exists and has the role 'READER' or higher
        let authorization = enforce_role_policy(&shared_state, &claims, UserRole::READER).await;

        match authorization {
            Ok(_authorized_user) => {
                let connection = shared_state.pool.get()
                    .expect("Failed to acquire connection from pool");

                match empiresTable::new(connection).get(empire_id) {
                    Ok(empire) => {
                        if let Some(empire) = empire {
                            Ok((StatusCode::OK, Json(empire)))
                        } else {
                            Err((StatusCode::NOT_FOUND, Json(json!({"error": "empire not found"}))))
                        }
                    },
                    Err(err) => {
                        eprintln!("Error reading empire: {:?}", err);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read empire"}))))
                    }
                }
            }
            Err(err) => Err(err)
        }
    }

    pub async fn update_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
        Json(upsert_empire): Json<UpsertEmpire>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;

        // Decode claims from bearer token header
        let claims = match decode_claims(&headers) {
            Ok(claims) => claims,
            Err((status_code, json_value)) => return Err((status_code, json_value)),
        };

        // Ensure that the user derived from claims exists and has the role 'EDITOR' or higher
        let authorization = enforce_role_policy(&shared_state, &claims, UserRole::EDITOR).await;

        match authorization {
            Ok(_authorized_user) => {
                let connection = shared_state.pool.get()
                    .expect("Failed to acquire connection from pool");

                match empiresTable::new(connection).update(empire_id, upsert_empire) {
                    Ok(updated_empire) => Ok((StatusCode::OK, Json(updated_empire))),
                    Err(diesel::result::Error::NotFound) => {
                        Err((StatusCode::NOT_FOUND, Json(json!({"error": "empire not found"}))))
                    },
                    Err(err) => {
                        eprintln!("Error updating empire: {:?}", err);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update empire"}))))
                    }
                }
            }
            Err(err) => Err(err)
        }
    }

    pub async fn delete_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;

        // Decode claims from bearer token header
        let claims = match decode_claims(&headers) {
            Ok(claims) => claims,
            Err((status_code, json_value)) => return Err((status_code, json_value)),
        };

        // Ensure that the user derived from claims exists and has the role 'ADMIN'
        let authorization = enforce_role_policy(&shared_state, &claims, UserRole::ADMIN).await;

        match authorization {
            Ok(_authorized_user) => {
                let connection = shared_state.pool.get()
                    .expect("Failed to acquire connection from pool");

                match empiresTable::new(connection).delete(empire_id) {
                    Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
                    Err(err) => {
                        eprintln!("Error deleting empire: {:?}", err);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to delete empire"}))))
                    }
                }
            }
            Err(err) => Err(err)
        }
    }
}
