use super::{DBBackendConnection, DBConnection};

use crate::database::{
    repository::{repository::get_connection, CentralSyncCursorRepository, RepositoryError},
    schema::CentralSyncBufferRow,
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

pub struct CentralSyncBufferRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl CentralSyncBufferRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> CentralSyncBufferRepository {
        CentralSyncBufferRepository { pool }
    }

    pub async fn insert_one_and_update_cursor(
        &self,
        central_sync_buffer_row: &CentralSyncBufferRow,
    ) -> Result<(), RepositoryError> {
        let connection = get_connection(&self.pool)?;
        let cursor = central_sync_buffer_row.id as u32;
        connection.transaction(|| {
            CentralSyncBufferRepository::insert_one_tx(&connection, central_sync_buffer_row)?;
            CentralSyncCursorRepository::update_cursor_tx(&connection, cursor)?;
            Ok(())
        })
    }

    pub async fn insert_one(
        &self,
        central_sync_buffer_row: &CentralSyncBufferRow,
    ) -> Result<(), RepositoryError> {
        let connection = get_connection(&self.pool)?;
        CentralSyncBufferRepository::insert_one_tx(&connection, central_sync_buffer_row)
    }

    pub fn insert_one_tx(
        connection: &DBConnection,
        central_sync_buffer_row: &CentralSyncBufferRow,
    ) -> Result<(), RepositoryError> {
        use crate::database::schema::diesel_schema::central_sync_buffer::dsl::*;
        diesel::insert_into(central_sync_buffer)
            .values(central_sync_buffer_row)
            .execute(connection)?;
        Ok(())
    }

    pub async fn pop_one(&self) -> Result<CentralSyncBufferRow, RepositoryError> {
        use crate::database::schema::diesel_schema::central_sync_buffer::dsl::*;
        let connection = get_connection(&self.pool)?;
        let result = central_sync_buffer.order(id.asc()).first(&connection)?;
        Ok(result)
    }

    pub async fn remove_all(&self) -> Result<(), RepositoryError> {
        use crate::database::schema::diesel_schema::central_sync_buffer::dsl::*;
        let connection = get_connection(&self.pool)?;
        diesel::delete(central_sync_buffer).execute(&connection)?;
        Ok(())
    }
}
