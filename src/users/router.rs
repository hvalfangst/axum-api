pub mod router {
    use std::{
        sync::Arc,
        time::Instant,
    };
    use serde_json::{json, Value};
    use bcrypt::{ hash, verify};
    use axum::{extract, extract::State, http::StatusCode, Json, response::IntoResponse, Router};
    use crate::{
        db::ConnectionPool,
        users::{
            service::service::DbExecutor,
            model::UpsertUser,
            model::LoginUser,
            model::Claims
        },
    };
    use std::collections::BTreeMap;
    use std::time::{Duration, SystemTime};
    use jsonwebtoken::{encode, EncodingKey, Header};


    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn users_routes(shared_connection_pool: ConnectionPool) -> Router {
        Router::new()
            .route("/users", axum::routing::post(create_user_handler))
            .route("/users/:user_id", axum::routing::get(read_user_handler))
            .route("/users/:user_id", axum::routing::put(update_user_handler))
            .route("/users/:user_id", axum::routing::delete(delete_user_handler))
            .route("/users/login", axum::routing::post(login_user_handler))
            .with_state(shared_connection_pool)
    }


    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn create_user_handler(
        State(shared_state): State<ConnectionPool>,
        Json(mut body): Json<UpsertUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

        // Validate format of field 'email'
        if !body.is_valid_email() {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": "Invalid input for field 'email'"}))))
        }

        let hash_pw_begin = Instant::now();

        // Hash password derived from request body
        if let Ok(hashed_password) = hash(&body.password, 12) {
            body.password = hashed_password;
        } else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to hash password"}))));
        }

        let hash_pw_end = hash_pw_begin.elapsed();
        println!("Time to hash pw: {:?}", hash_pw_end);

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = DbExecutor::new(connection);

        let create_user_begin = Instant::now();

        if let Err(err) = users.create(body.clone()) {
            eprintln!("Create user failed: {:?}", err);
            return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": "Failed to create user"}))));
        }

        let create_user_end = create_user_begin.elapsed();
        println!("Time to create user: {:?}", create_user_end);

        Ok((StatusCode::CREATED, Json(body)))
    }

    pub async fn read_user_handler(
        State(shared_state): State<ConnectionPool>,
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
        State(shared_state): State<ConnectionPool>,
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
        State(shared_state): State<ConnectionPool>,
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



    pub async fn login_user_handler(
        State(shared_state): State<ConnectionPool>,
        Json(body): Json<LoginUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = DbExecutor::new(connection);

        match users.readByEmail(body.email.clone()) {
            Ok(Some(user)) if body.email == user.email => {

                // Verify the hashed password
                if verify(&body.password, &user.password).unwrap_or(false) {

                    // Fetch the user's role based on the roleId
                    let role = match users.fetchRoleByRoleId(user.role_id) {
                        Ok(Some(role)) => role,
                        _ => {
                            return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Role not found"}))));
                        }
                    };


                    let expiration = SystemTime::now()
                        .checked_add(Duration::from_secs(3600)) // Set the token to expire in 1 hour
                        .expect("Failed to calculate token expiration")
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("SystemTime before UNIX EPOCH")
                        .as_secs() as i64;

                    let claims = Claims {
                        sub: user.email.clone(),
                        role: role.role_name.clone(),
                        exp: expiration
                    };

                    // Generate and sign the token
                    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();


                    Ok((StatusCode::OK, Json(token)))
                } else {
                    Err((StatusCode::NOT_FOUND, Json(json!({"error": "Wrong password"}))))
                }
            },
            Ok(Some(_)) => {
                Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))))
            },
            Ok(None) => {
                Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))))
            },
            Err(err) => {
                eprintln!("Error reading user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read user"}))))
            }
        }
     }

    #[cfg(test)]
    mod tests {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use serde_json::json;
        use tower::ServiceExt;
        use crate::{create_shared_connection_pool, load_environment_variable, users_routes};
        use crate::users::model::UpsertUser;
        use crate::users::service::service::DbExecutor;

        #[tokio::test]
        async fn post_users_returns_201_on_valid_data() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let service = users_routes(connection_pool);

            let request_body = UpsertUser {
                email: "valid@email.com".to_string(),
                password: "Big100".to_string(),
                fullname: "Kenneth Molasses".to_string(),
                role_id: 1
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri("/users")
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
        async fn post_users_returns_422_on_invalid_email() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let service = users_routes(connection_pool);

            let request_body = UpsertUser {
                email: "eg-klare-meg".to_string(),
                password: "Big100".to_string(),
                fullname: "Kenneth Molasses".to_string(),
                role_id: 1
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri("/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 422
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        }

        #[tokio::test]
        async fn put_users_returns_200_on_valid_data() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);
            let service = users_routes(connection_pool);

            // Data
            let request_body = UpsertUser {
                email: "ernst@snowmail.com".to_string(),
                password: "feltedsnowmen".to_string(),
                fullname: "Ernst van Schnee".to_string(),
                role_id: 1
            };

            // Create a new location with the above data
            let created_user = db_executor.create(request_body.clone()).expect("Create location failed");

            // Assert equality
            assert_eq!(request_body.email, created_user.email);
            assert_eq!(request_body.password, created_user.password);
            assert_eq!(request_body.fullname, created_user.fullname);
            assert_eq!(request_body.role_id, created_user.role_id);

            // Data
            let updated_request_body = UpsertUser {
                email: "ernst@snowmail.com".to_string(),
                password: "feltseng?".to_string(),
                fullname: "Ernst van Schnee".to_string(),
                role_id: 1
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri(format!("/users/{}", created_user.id))
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
                "id": created_user.id,
                "email": updated_request_body.email,
                "password": updated_request_body.password,
                "fullname": updated_request_body.fullname,
                "role_id": updated_request_body.role_id
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_users_returns_200_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);
            let service = users_routes(connection_pool);

            let request_body = UpsertUser {
                email: "glossy@ringdue.no".to_string(),
                password: "LillePostBudMin".to_string(),
                fullname: "Glossy Garnished".to_string(),
                role_id: 1
            };

            // Create a new location with the above data
            let created_user = db_executor.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/users/{}", created_user.id))
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
                "id": created_user.id,
                "email": request_body.email,
                "password": request_body.password,
                "fullname": request_body.fullname,
                "role_id": request_body.role_id
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_users_returns_404_on_non_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let service = users_routes(connection_pool);

            // Create a request with the aforementioned id
            let request = Request::builder()
                .uri(format!("/users/{}", -666)) // Use a non-existent ID
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
        async fn delete_users_returns_204() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);
            let service = users_routes(connection_pool);

            let request_body = UpsertUser {
                email: "josek@ifi.uio.no".to_string(),
                password: "TurboPascalLife".to_string(),
                fullname: "Jose Kernelio".to_string(),
                role_id: 1
            };

            // Create a new user with the above data
            let created_user = db_executor.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/users/{}", created_user.id))
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

            // Attempt to retrieve the deleted user
            let deleted_user_result = db_executor.read(created_user.id);

            // Assert that the Result is Ok (no error)
            assert!(deleted_user_result.is_ok());

            // Extract the Option<User> from the Ok variant
            let deleted_user = deleted_user_result.unwrap();

            // Assert that the deleted user is None (i.e., it doesn't exist)
            assert!(deleted_user.is_none());
        }
    }
}
