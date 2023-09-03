pub mod service {
    use diesel::{
        prelude::*,
        PgConnection,
        r2d2::{ConnectionManager, PooledConnection},
    };
    use crate::{
        users::model::{User, UpsertUser},
        schema
    };

    type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

    pub struct DbExecutor {
        connection: PooledPg,
    }

    impl DbExecutor {
        pub fn new(connection: PooledPg) -> DbExecutor {
            DbExecutor { connection }
        }

        pub fn create(&mut self, create_user: UpsertUser) -> Result<User, diesel::result::Error> {
            use schema::users;

            let new_user = diesel::insert_into(users::table)
                .values((
                    users::email.eq(&create_user.email),
                    users::password.eq(&create_user.password),
                    users::fullname.eq(&create_user.fullname),
                    users::role_id.eq(&create_user.role_id),
                ))
                .get_result(&mut self.connection).expect("Create user failed");

            Ok(new_user)
        }

        pub fn read(&mut self, user_id: i32) -> Result<Option<User>, diesel::result::Error> {
            use schema::users;

            let user = users::table.find(user_id)
                .get_result(&mut self.connection)
                .optional()?;

            Ok(user)
        }

        pub fn update(&mut self, user_id: i32, update_user: UpsertUser) -> Result<User, diesel::result::Error> {
            use schema::users;

            // Check if the user exists before attempting to update
            let existing_user = users::table.find(user_id)
                .get_result::<User>(&mut self.connection);

            match existing_user {
                Ok(_) => {
                    let updated_user = diesel::update(users::table.find(user_id))
                        .set((
                            users::email.eq(&update_user.email),
                            users::password.eq(&update_user.password),
                            users::fullname.eq(&update_user.fullname),
                            users::role_id.eq(&update_user.role_id),
                        ))
                        .get_result(&mut self.connection)
                        .expect("Update user failed");

                    Ok(updated_user)
                },
                Err(_) => Err(diesel::result::Error::NotFound)
            }
        }


        pub fn delete(&mut self, user_id: i32) -> Result<(), diesel::result::Error> {
            use schema::users;

            // Check if the location exists before attempting to delete
            let existing_location = users::table.find(user_id)
                .get_result::<User>(&mut self.connection);

            match existing_location {
                Ok(_) => {
                    diesel::delete(users::table.find(user_id))
                        .execute(&mut self.connection)?;
                    Ok(())
                },
                Err(_) => {
                    Err(diesel::result::Error::NotFound)
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            create_shared_connection_pool,
            load_environment_variable,
            users::{
                model::UpsertUser,
                service::service::DbExecutor
            }
        };

        #[test]
        fn create_succeeds_on_valid_input() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);

            let new_user = UpsertUser {
                email: "obelisksx@ifi.uio.no".to_string(),
                password: "EatSleepRepeat".to_string(),
                fullname: "Obelix fra IFI".to_string(),
                role_id: 1
            };

            let created_user = db_executor.create(new_user.clone()).expect("Create user failed");

            assert_eq!(created_user.email, new_user.email);
            assert_eq!(created_user.password, new_user.password);
            assert_eq!(created_user.fullname, new_user.fullname);
            assert_eq!(created_user.role_id, new_user.role_id);
        }

        #[test]
        fn read_succeeds_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);

            let new_user = UpsertUser {
                email: "kokemakken@tremakk.no".to_string(),
                password: "huuuuuman".to_string(),
                fullname: "Woodwormius".to_string(),
                role_id: 1
            };

            let created_user = db_executor.create(new_user.clone()).expect("Create user failed");
            let retrieved_user = db_executor.read(created_user.id).expect("Read user failed").unwrap();

            assert_eq!(retrieved_user.email, new_user.email);
            assert_eq!(retrieved_user.password, new_user.password);
            assert_eq!(retrieved_user.fullname, new_user.fullname);
            assert_eq!(retrieved_user.role_id, new_user.role_id);
        }

        #[test]
        fn read_returns_none_on_non_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);

            let retrieved_user = db_executor.read(-666); // Use a non-existing ID

            assert!(retrieved_user.is_ok()); // Expecting Ok(none)
            assert!((retrieved_user.unwrap().is_none()));
        }

        #[test]
        fn update_succeeds_on_valid_input() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);

            let original_request = UpsertUser {
                email: "pondi@wwf.com".to_string(),
                password: "SnorkSnorkSnork".to_string(),
                fullname: "Panda Pondi".to_string(),
                role_id: 1
            };

            let original_user = db_executor.create(original_request.clone()).expect("Create user failed");

            let updated_request = UpsertUser {
                email: "uhi@wwf.com".to_string(),
                password: "SlafsSlafsSlaf".to_string(),
                fullname: "Panda Pondi".to_string(),
                role_id: 1
            };

            let updated_user = db_executor.update(original_user.id, updated_request.clone()).expect("Update user failed");

            assert_eq!(updated_user.email, updated_request.email);
            assert_eq!(updated_user.password, updated_request.password);
            assert_eq!(updated_user.fullname, updated_request.fullname);
            assert_eq!(updated_user.role_id, updated_request.role_id);
        }

        #[test]
        fn update_fails_on_nonexistent_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);

            let request = UpsertUser {
                email: "lukewarm@manlet.com".to_string(),
                password: "realfrogeyes".to_string(),
                fullname: "Lukas Parrot".to_string(),
                role_id: 1
            };

            let result = db_executor.update(-666, request.clone());  // Use a non-existent ID

            assert!(result.is_err());  // Expecting an error as the ID is not present
        }

        #[test]
        fn delete_succeeds_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);

            let request = UpsertUser {
                email: "world.according.to.jesse@mongols.com".to_string(),
                password: "bunchofslackjawedfgets".to_string(),
                fullname: "Jesse Ventura".to_string(),
                role_id: 1
            };

            let user = db_executor.create(request.clone()).expect("Create user failed");
            db_executor.delete(user.id.clone()).expect("Delete user failed");
            let deleted_user = db_executor.read(user.id).expect("Read user failed");

            assert!(deleted_user.is_none()); // Expecting lack of value as user has been deleted
        }

        #[test]
        fn delete_fails_on_nonexistent_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut db_executor = DbExecutor::new(connection);
            let result = db_executor.delete(-666);  // Use a non-existent ID

            assert!(result.is_err());  // Expecting an error as the ID is not present
        }
    }
}

