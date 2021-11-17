use super::StorageConnection;

use crate::repository_error::RepositoryError;
use crate::schema::diesel_schema::location::dsl as location_dsl;
use crate::schema::LocationRow;

use diesel::prelude::*;

pub struct LocationRowRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> LocationRowRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        LocationRowRepository { connection }
    }

    #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
    pub fn upsert_one(&self, row: &LocationRow) -> Result<(), RepositoryError> {
        diesel::insert_into(location_dsl::location)
            .values(row)
            .on_conflict(id)
            .do_update()
            .set(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    pub fn upsert_one(&self, row: &LocationRow) -> Result<(), RepositoryError> {
        diesel::replace_into(location_dsl::location)
            .values(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub fn find_one_by_id(&self, id: &str) -> Result<LocationRow, RepositoryError> {
        location_dsl::location
            .filter(location_dsl::id.eq(id))
            .first(&self.connection.connection)
            .map_err(RepositoryError::from)
    }
}