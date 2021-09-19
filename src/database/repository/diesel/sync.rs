use super::{
    get_connection, master_list::MasterListRepository, master_list_line::MasterListLineRepository,
    master_list_name_join::MasterListNameJoinRepository, DBBackendConnection, DBConnection,
    ItemRepository, NameRepository, StoreRepository,
};

use crate::database::{
    repository::RepositoryError,
    schema::{ItemRow, MasterListLineRow, MasterListNameJoinRow, MasterListRow, NameRow, StoreRow},
};

use std::future::Future;

use diesel::{
    connection::TransactionManager,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

pub enum IntegrationUpsertRecord {
    Name(NameRow),
    Item(ItemRow),
    Store(StoreRow),
    MasterList(MasterListRow),
    MasterListLine(MasterListLineRow),
    MasterListNameJoin(MasterListNameJoinRow),
}

pub struct IntegrationRecord {
    pub upserts: Vec<IntegrationUpsertRecord>,
}

pub struct SyncRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

pub struct SyncSession {
    tx: DBConnection,
}

impl SyncSession {
    pub async fn transaction<'a, F, Fut>(&'a self, f: F) -> Result<(), String>
    where
        F: FnOnce(&'a SyncSession) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let transaction_manager = self.tx.transaction_manager();
        transaction_manager
            .begin_transaction(&self.tx)
            .map_err(|_| "Failed to start tx".to_string())?;
        match f(&self).await {
            Ok(value) => {
                transaction_manager
                    .commit_transaction(&self.tx)
                    .map_err(|_| "Failed to end tx".to_string())?;
                Ok(value)
            }
            Err(e) => {
                transaction_manager
                    .rollback_transaction(&self.tx)
                    .map_err(|_| "Failed to rollback tx".to_string())?;
                Err(e)
            }
        }
    }
}

impl SyncRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> SyncRepository {
        SyncRepository { pool }
    }

    /// Creates a sync session.
    ///
    /// All IntegrationRecord added in the same session are added in a single storage transaction,
    /// i.e. if the integration fails nothing is added to the database.
    pub async fn new_sync_session(&self) -> Result<SyncSession, RepositoryError> {
        Ok(SyncSession {
            tx: get_connection(&self.pool)?,
        })
    }

    pub async fn integrate_records(
        &self,
        session: &SyncSession,
        integration_records: &IntegrationRecord,
    ) -> Result<(), RepositoryError> {
        let tx = &session.tx;
        for record in &integration_records.upserts {
            match &record {
                IntegrationUpsertRecord::Name(record) => {
                    NameRepository::upsert_one_tx(&tx, record)?
                }
                IntegrationUpsertRecord::Item(record) => {
                    ItemRepository::upsert_one_tx(&tx, record)?
                }
                IntegrationUpsertRecord::Store(record) => {
                    StoreRepository::upsert_one_tx(&tx, record)?
                }
                IntegrationUpsertRecord::MasterList(record) => {
                    MasterListRepository::upsert_one_tx(&tx, record)?
                }
                IntegrationUpsertRecord::MasterListLine(record) => {
                    MasterListLineRepository::upsert_one_tx(&tx, record)?
                }
                IntegrationUpsertRecord::MasterListNameJoin(record) => {
                    MasterListNameJoinRepository::upsert_one_tx(&tx, record)?
                }
            }
        }
        Ok(())
    }
}
