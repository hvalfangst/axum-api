pub mod router {
    use std::borrow::Borrow;
    use std::ops::Deref;
    use std::sync::Arc;
    use std::thread;
    use serde_json::{json, Value};
    use axum::{Router, http::StatusCode, Json, response::IntoResponse, extract::State, extract};
    use axum::http::header::{HeaderName, HeaderValue, AUTHORIZATION};
    use axum::middleware::Next;
    use diesel::result::Error;
    use http::{HeaderMap, Request, Response};
    use jsonwebtoken::{Algorithm, decode, DecodingKey, TokenData, Validation};
    use crate::{
        db::ConnectionPool,
        users::service::service::DbExecutor as UsersDB,
        locations:: {
            service::service::DbExecutor as locationsDB,
            model::UpsertLocation
        },
    };
    use crate::users::model::{Claims, User};
    use jsonwebtoken::errors::ErrorKind as JwtErrorKind;

    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn locations_routes(shared_connection_pool: ConnectionPool) -> Router {
        Router::new()
            .route("/locations", axum::routing::post(create_location_handler))
            .route("/locations/:location_id", axum::routing::get(read_location_handler))
            .route("/locations/:location_id", axum::routing::put(update_location_handler))
            .route("/locations/:location_id", axum::routing::delete(delete_location_handler))
            .with_state(shared_connection_pool)
    }


    fn role_to_string(role: i32) -> String {
        match role {
            1 => "READER".to_string(),
            2 => "WRITER".to_string(),
            3 => "EDITOR".to_string(),
            4 => "ADMIN".to_string(),
            _ => "INVALID_ROLE".to_string(),
        }
    }

    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn create_location_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        Json(upsert_location): Json<UpsertLocation>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

        // Decode claims from bearer token header
        let claims = match decode_claims(&headers) {
            Ok(claims) => claims,
            Err((status_code, json_value)) => return Err((status_code, json_value)),
        };

        // Ensure that the user derived from claims exists and has the role 'CREATOR'
        let authorization = enforce_role_policy(&shared_state, &claims, "CREATOR".to_string()).await;

        match authorization {
            Ok(authorized_user) => {
                let connection = shared_state.pool.get().expect("Failed to acquire connection from pool");
                let mut locations = locationsDB::new(connection);

                match  locations.create(upsert_location) {
                    Ok(new_location) => Ok((StatusCode::CREATED, Json(new_location))),
                    Err(err) => {
                        eprintln!("Error creating location: {:?}", err);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create location"}))))
                    }
                }
            }
            Err(err) => Err(err),
        }
    }

    fn decode_claims(headers: &HeaderMap) -> Result<Option<TokenData<Claims>>, (StatusCode, Json<Value>)> {

        // Retrieve Authorization header from the map of request headers
        let token_header = headers.get("Authorization");

        // Map token if it exists - return error if not
        let token = match token_header {
            None => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Missing header"})),
                ));
            }
            Some(header) => header.to_str().unwrap(),
        };

        eprintln!("Token: {:?}", token);

        // Return error if the the token does not start with "Bearer"
        if !token.starts_with("Bearer ") {
            eprintln!("Token is missing 'Bearer ' prefix");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Token is missing 'Bearer ' prefix"})),
            ));
        }

        // Attempt to decode token and match the results
        match decode::<Claims>(
            &token[7..],
            &DecodingKey::from_secret("secret".as_ref()),
            &Validation::new(Algorithm::HS256),
        ) {
            Err(err) => {
                match err.kind() {
                    // Handle the specific ExpiredSignature error
                    JwtErrorKind::ExpiredSignature => {
                        eprintln!("JWT expired: {:?}", err);
                        Err((
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "Token has expired"})),
                        ))
                    }
                    _ => {
                        // Handle other decoding errors
                        eprintln!("Error decoding JWT: {:?}", err);
                        Err((
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "Invalid JWT"})),
                        ))
                    }
                }
            }
            Ok(decoded_claims) => Ok(Some(decoded_claims)),
        }
    }

    async fn enforce_role_policy(
        shared_state: &ConnectionPool,
        claims: &Option<TokenData<Claims>>,
        required_role: String,
    ) -> Result<Option<User>, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get().expect("Failed to acquire connection from pool");
        let mut users = UsersDB::new(connection);

        match users.readByEmail(claims.clone().unwrap().claims.sub) {
            Ok(user) => {
                let user_role = role_to_string(user.clone().unwrap().role_id);
                let claims_role = claims.clone().unwrap().claims.role;

                if user_role != required_role {
                    eprintln!("User role: {} does not match required role: {}", user_role, required_role);
                    Err((StatusCode::UNAUTHORIZED,
                         Json(json!({"error": format!("Current role of {} does not match the required role of {}", user_role, required_role)}))))
                } else if user_role != claims_role {
                    eprintln!("Role in claims, {} does not match role in DB: {}", claims_role, user_role);
                    Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "Roles in claims differ from DB"}))))
                } else {
                    Ok(user)
                }
            }
            Err(err) => {
                eprintln!("User in claims not found in DB {:?}", err);
                Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "User in claims not found in DB"}))))
            }
        }
    }

    pub async fn read_location_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut locations = locationsDB::new(connection);

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
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32,)>,
        Json(upsert_location): Json<UpsertLocation>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut locations = locationsDB::new(connection);

        match locations.update(location_id, upsert_location) {
            Ok(updated_location) => Ok((StatusCode::OK, Json(updated_location))),
            Err(diesel::result::Error::NotFound) => {
                Err((StatusCode::NOT_FOUND, Json(json!({"error": "Location not found"}))))
            },
            Err(err) => {
                eprintln!("Error updating location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update location"}))))
            }
        }
    }

    pub async fn delete_location_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut locations = locationsDB::new(connection);

        match locations.delete(location_id) {
            Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
            Err(err) => {
                eprintln!("Error deleting location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to delete location"}))))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use serde_json::json;
        use tower::ServiceExt;
        use crate::{create_shared_connection_pool, load_environment_variable, locations_routes};
        use crate::locations::model::UpsertLocation;
        use crate::locations::service::service::DbExecutor;

        #[tokio::test]
        async fn post_locations_returns_201_on_valid_data() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let service = locations_routes(connection_pool);

            // Data
            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri("/locations")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 201
            assert_eq!(response.status(), StatusCode::CREATED);
        }

        #[tokio::test]
        async fn put_locations_returns_200_on_valid_data() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);
            let service = locations_routes(connection_pool);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = db_executor.create(request_body.clone()).expect("Create location failed");

            // Assert equality
            assert_eq!(request_body.star_system, created_location.star_system);
            assert_eq!(request_body.area, created_location.area);

            let updated_request_body = UpsertLocation {
                star_system: "Kador".to_string(),
                area: "The Crimson Expanse".to_string(),
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 200
            assert_eq!(response.status(), StatusCode::OK);

            // Extract body from response
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Construct JSON consisting of expected payload
            let expected_response = json!({
                "id": created_location.id,
                "area": updated_request_body.area,
                "star_system": updated_request_body.star_system
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_locations_returns_200_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);
            let service = locations_routes(connection_pool);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = db_executor.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("GET")
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 200
            assert_eq!(response.status(), StatusCode::OK);

            // Extract body from response
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Construct JSON consisting of expected payload
            let expected_response = json!({
                "id": created_location.id,
                "area": request_body.area,
                "star_system": request_body.star_system
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_locations_returns_404_on_non_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let service = locations_routes(connection_pool);

            // Create a request with the aforementioned id
            let request = Request::builder()
                .uri(format!("/locations/{}", -666)) // Use a non-existent ID
                .method("GET")
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 404 as there are no locations associated with the id
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn delete_locations_returns_204() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);
            let service = locations_routes(connection_pool);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = db_executor.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("DELETE")
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 204
            assert_eq!(response.status(), StatusCode::NO_CONTENT);

            // Attempt to retrieve the deleted location
            let deleted_location_result = db_executor.read(created_location.id);

            // Assert that the Result is Ok (no error)
            assert!(deleted_location_result.is_ok());

            // Extract the Option<Location> from the Ok variant
            let deleted_location = deleted_location_result.unwrap();

            // Assert that the deleted location is None (i.e., it doesn't exist)
            assert!(deleted_location.is_none());
        }
    }
}
