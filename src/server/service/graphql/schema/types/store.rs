use crate::{
    database::{
        loader::NameLoader,
        repository::CustomerInvoiceRepository,
        schema::{NameRow, StoreRow, TransactRow},
    },
    server::service::graphql::{
        schema::types::{Name, Transact},
        ContextExt,
    },
};

use async_graphql::{dataloader::DataLoader, Context, Object};

#[derive(Clone)]
pub struct Store {
    pub store_row: StoreRow,
}

#[Object]
impl Store {
    pub async fn id(&self) -> &str {
        &self.store_row.id
    }

    pub async fn name(&self, ctx: &Context<'_>) -> Name {
        let name_loader = ctx.get_loader::<DataLoader<NameLoader>>();

        let name_row: NameRow = name_loader
            .load_one(self.store_row.name_id.clone())
            .await
            .unwrap_or_else(|_| panic!("Failed to get name for store {}", self.store_row.id))
            .ok_or_else(|| panic!("Failed to get name for store {}", self.store_row.id))
            .unwrap_or_else(|_| panic!("Failed to get name for store {}", self.store_row.id));

        Name { name_row }
    }

    pub async fn customer_invoices(&self, ctx: &Context<'_>) -> Vec<Transact> {
        let customer_invoice_repository = ctx.get_repository::<CustomerInvoiceRepository>();

        let customer_invoice_rows: Vec<TransactRow> = customer_invoice_repository
            .find_many_by_store_id(&self.store_row.id)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get customer invoices for store {}",
                    self.store_row.id
                )
            });

        customer_invoice_rows
            .into_iter()
            .map(|customer_invoice_row| Transact {
                transact_row: customer_invoice_row,
            })
            .collect()
    }
}
