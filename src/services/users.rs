pub mod service {
    use std::process::id;
    use diesel::prelude::*;
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::PgConnection;
    use crate::{model::{UpsertUser, User}, schema, users};

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

        pub fn read(&mut self, user_id: i32) -> Result<Vec<User>, diesel::result::Error> {
            use schema::users;

            let vector = users::table.filter(users::user_id.eq(user_id))
                .load::<User>(&mut self.connection)
                .expect("Failed loading user");

            Ok(vector)
        }

        pub fn update(&mut self, user_id: i32, update_user: UpsertUser) -> Result<User, diesel::result::Error> {
            use schema::users;

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
        }

        pub fn delete(&mut self, user_id: i32) -> Result<(), diesel::result::Error> {
            use schema::users;

            diesel::delete(users::table.find(user_id))
                .execute(&mut self.connection)?;

            Ok(())
        }
    }
}

