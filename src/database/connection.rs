use crate::database::mocks;
use crate::database::queries;
use crate::database::schema::{
    ItemLineRow, ItemRow, NameRow, RequisitionLineRow, RequisitionRow, StoreRow, TransactLineRow,
    TransactRow,
};

#[derive(Clone)]
pub struct DatabaseConnection {
    pool: sqlx::PgPool,
}

impl DatabaseConnection {
    pub async fn new(pool: sqlx::PgPool) -> DatabaseConnection {
        DatabaseConnection { pool }
    }

    pub async fn insert_mock_data(&self) -> Result<(), sqlx::Error> {
        self.create_names(&mocks::mock_names())
            .await
            .expect("Failed to insert mock name data");

        self.create_stores(&mocks::mock_stores())
            .await
            .expect("Failed to insert mock store data");

        self.create_items(&mocks::mock_items())
            .await
            .expect("Failed to insert mock item data");

        self.create_item_lines(&mocks::mock_item_lines())
            .await
            .expect("Failed to insert mock item line data");

        self.create_requisitions(&mocks::mock_requisitions())
            .await
            .expect("Failed to insert mock requisition data");

        self.create_requisition_lines(&mocks::mock_requisition_lines())
            .await
            .expect("Failed to insert mock requisition line data");

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn create_store(&self, store: &StoreRow) -> Result<(), sqlx::Error> {
        queries::insert_store(&self.pool, store).await
    }

    #[allow(dead_code)]
    pub async fn create_stores(&self, stores: &[StoreRow]) -> Result<(), sqlx::Error> {
        queries::insert_stores(&self.pool, stores).await
    }

    #[allow(dead_code)]
    pub async fn create_name(&self, name: &NameRow) -> Result<(), sqlx::Error> {
        queries::insert_name(&self.pool, name).await
    }

    #[allow(dead_code)]
    pub async fn create_names(&self, names: &[NameRow]) -> Result<(), sqlx::Error> {
        queries::insert_names(&self.pool, names).await
    }

    pub async fn create_item(&self, item: &ItemRow) -> Result<(), sqlx::Error> {
        queries::insert_item(&self.pool, item).await
    }

    pub async fn create_items(&self, items: &[ItemRow]) -> Result<(), sqlx::Error> {
        queries::insert_items(&self.pool, items).await
    }

    #[allow(dead_code)]
    pub async fn create_item_line(&self, item_line: &ItemLineRow) -> Result<(), sqlx::Error> {
        queries::insert_item_line(&self.pool, item_line).await
    }

    #[allow(dead_code)]
    pub async fn create_item_lines(&self, item_lines: &[ItemLineRow]) -> Result<(), sqlx::Error> {
        queries::insert_item_lines(&self.pool, item_lines).await
    }

    pub async fn create_requisition(
        &self,
        requisition: &RequisitionRow,
    ) -> Result<(), sqlx::Error> {
        queries::insert_requisition(&self.pool, requisition).await
    }

    pub async fn create_requisitions(
        &self,
        requisitions: &[RequisitionRow],
    ) -> Result<(), sqlx::Error> {
        queries::insert_requisitions(&self.pool, requisitions).await
    }

    pub async fn create_requisition_line(
        &self,
        requisition_line: &RequisitionLineRow,
    ) -> Result<(), sqlx::Error> {
        queries::insert_requisition_line(&self.pool, requisition_line).await
    }

    pub async fn create_requisition_lines(
        &self,
        requisition_lines: &[RequisitionLineRow],
    ) -> Result<(), sqlx::Error> {
        queries::insert_requisition_lines(&self.pool, requisition_lines).await
    }

    pub async fn get_store(&self, id: &str) -> Result<StoreRow, sqlx::Error> {
        queries::select_store(&self.pool, id).await
    }

    pub async fn get_name(&self, id: &str) -> Result<NameRow, sqlx::Error> {
        queries::select_name(&self.pool, id).await
    }

    pub async fn get_item(&self, id: &str) -> Result<ItemRow, sqlx::Error> {
        queries::select_item(&self.pool, id).await
    }

    pub async fn get_item_line(&self, id: &str) -> Result<ItemLineRow, sqlx::Error> {
        queries::select_item_line(&self.pool, id).await
    }

    pub async fn get_requisition(&self, id: &str) -> Result<RequisitionRow, sqlx::Error> {
        queries::select_requisition(&self.pool, id).await
    }

    #[allow(dead_code)]
    pub async fn get_requisition_line(&self, id: &str) -> Result<RequisitionLineRow, sqlx::Error> {
        queries::select_requisition_line(&self.pool, id).await
    }

    pub async fn get_requisition_lines(
        &self,
        requisition_id: &str,
    ) -> Result<Vec<RequisitionLineRow>, sqlx::Error> {
        queries::select_requisition_lines(&self.pool, requisition_id).await
    }

    #[allow(dead_code)]
    pub async fn get_transact(&self, id: &str) -> Result<TransactRow, sqlx::Error> {
        queries::select_transact(&self.pool, id).await
    }

    #[allow(dead_code)]
    pub async fn get_transacts(&self, name_id: &str) -> Result<Vec<TransactRow>, sqlx::Error> {
        queries::select_transacts(&self.pool, name_id).await
    }

    #[allow(dead_code)]
    pub async fn get_transact_line(&self, id: &str) -> Result<TransactLineRow, sqlx::Error> {
        queries::select_transact_line(&self.pool, id).await
    }

    #[allow(dead_code)]
    pub async fn get_transact_lines(
        &self,
        transact_id: &str,
    ) -> Result<Vec<TransactLineRow>, sqlx::Error> {
        queries::select_transact_lines(&self.pool, transact_id).await
    }
}
