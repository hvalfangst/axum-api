pub mod service {
    use std::process::id;
    use diesel::prelude::*;
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::PgConnection;
    use crate::{model::{UpsertLocation, Location}, schema};
    use crate::services::locations;

    type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

    pub struct DbExecutor {
        connection: PooledPg,
    }

    impl DbExecutor {
        pub fn new(connection: PooledPg) -> DbExecutor {
            DbExecutor { connection }
        }

        pub fn create(&mut self, upsert_location: UpsertLocation) -> Result<Location, diesel::result::Error> {
            use schema::locations;

            let new_location = diesel::insert_into(locations::table)
                .values((
                    locations::star_system.eq(&upsert_location.star_system),
                    locations::area.eq(&upsert_location.area),
                ))
                .get_result(&mut self.connection)
                .expect("Create location failed");

            Ok(new_location)
        }

        pub fn read(&mut self, location_id: i32) -> Result<Option<Location>, diesel::result::Error> {
            use schema::locations;

            let location = locations::table.find(location_id)
                .get_result(&mut self.connection)
                .optional()?;

            Ok(location)
        }

        pub fn update(&mut self, location_id: i32, upsert_location: UpsertLocation) -> Result<Location, diesel::result::Error> {
            use schema::locations;

            let updated_location = diesel::update(locations::table.find(location_id))
                .set((
                    locations::star_system.eq(&upsert_location.star_system),
                    locations::area.eq(&upsert_location.area),
                ))
                .get_result(&mut self.connection)
                .expect("Update location failed");

            Ok(updated_location)
        }

        pub fn delete(&mut self, location_id: i32) -> Result<(), diesel::result::Error> {
            use schema::locations;

            diesel::delete(locations::table.find(location_id))
                .execute(&mut self.connection)?;

            Ok(())
        }

    }
}

