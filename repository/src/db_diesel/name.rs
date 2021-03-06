use super::StorageConnection;

use crate::{repository_error::RepositoryError, schema::NameRow};

use diesel::prelude::*;

pub struct NameRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> NameRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        NameRepository { connection }
    }

    #[cfg(feature = "postgres")]
    pub fn upsert_one(&self, name_row: &NameRow) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::name::dsl::*;
        diesel::insert_into(name)
            .values(name_row)
            .on_conflict(id)
            .do_update()
            .set(name_row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(not(feature = "postgres"))]
    pub fn upsert_one(&self, name_row: &NameRow) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::name::dsl::*;
        diesel::replace_into(name)
            .values(name_row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub async fn insert_one(&self, name_row: &NameRow) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::name::dsl::*;
        diesel::insert_into(name)
            .values(name_row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, name_id: &str) -> Result<NameRow, RepositoryError> {
        use crate::schema::diesel_schema::name::dsl::*;
        let result = name
            .filter(id.eq(name_id))
            .first(&self.connection.connection)?;
        Ok(result)
    }

    pub fn find_many_by_id(&self, ids: &[String]) -> Result<Vec<NameRow>, RepositoryError> {
        use crate::schema::diesel_schema::name::dsl::*;
        let result = name
            .filter(id.eq_any(ids))
            .load(&self.connection.connection)?;
        Ok(result)
    }
}
