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

            diesel::delete(users::table.find(user_id))
                .execute(&mut self.connection)?;

            Ok(())
        }
    }
}

