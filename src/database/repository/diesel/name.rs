use crate::database::{
    repository::{
        macros::{execute_connection, first_pool, load_pool},
        DbConnection, DbConnectionPool, RepositoryError,
    },
    schema::{diesel_schema::name_table::dsl::*, NameRow},
};
use diesel::prelude::*;
pub struct NameRepository {
    pool: DbConnectionPool,
}

impl NameRepository {
    pub fn new(pool: DbConnectionPool) -> NameRepository {
        NameRepository { pool }
    }

    pub fn insert_one_tx(
        connection: &DbConnection,
        name_row: &NameRow,
    ) -> Result<(), RepositoryError> {
        execute_connection!(connection, diesel::insert_into(name_table).values(name_row))?;
        Ok(())
    }

    pub fn upsert_one_tx(
        connection: &DbConnection,
        name_row: &NameRow,
    ) -> Result<(), RepositoryError> {
        match connection {
            DbConnection::Pg(pg_connection) => diesel::insert_into(name_table)
                .values(name_row)
                .on_conflict(id)
                .do_update()
                .set(name_row)
                .execute(&**pg_connection),
            DbConnection::Sqlite(sqlite_connection) => diesel::replace_into(name_table)
                .values(name_row)
                .execute(&**sqlite_connection),
        }?;

        Ok(())
    }

    pub async fn insert_one(&self, name_row: &NameRow) -> Result<(), RepositoryError> {
        NameRepository::insert_one_tx(&self.pool.get_connection()?, name_row)?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, name_id: &str) -> Result<NameRow, RepositoryError> {
        first_pool!(self.pool, name_table.filter(id.eq(name_id)))
    }

    pub async fn find_many_by_id(&self, ids: &[String]) -> Result<Vec<NameRow>, RepositoryError> {
        load_pool!(self.pool, name_table.filter(id.eq_any(ids)))
    }
}
