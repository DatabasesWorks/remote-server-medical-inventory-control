use crate::{
    database::{repository::CustomerInvoiceRepository, schema::NameRow},
    server::service::graphql::{schema::types::Transact, ContextExt},
};

use async_graphql::{Context, Object};

#[derive(Clone)]
pub struct Name {
    pub name_row: NameRow,
}

#[Object]
impl Name {
    pub async fn id(&self) -> &str {
        &self.name_row.id
    }

    pub async fn name(&self) -> &str {
        &self.name_row.id
    }

    pub async fn customer_invoices(&self, ctx: &Context<'_>) -> Vec<Transact> {
        let customer_invoice_repository = ctx.get_repository::<CustomerInvoiceRepository>();

        let customer_invoice_rows = customer_invoice_repository
            .find_many_by_name_id(&self.name_row.id)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get customer invoices for name {}",
                    self.name_row.id
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
