use crate::{
    database::{
        loader::NameLoader,
        repository::TransactLineRepository,
        schema::{NameRow, TransactLineRow, TransactRow, TransactRowType},
    },
    server::service::graphql::{
        schema::types::{Name, TransactLine},
        ContextExt,
    },
};

use async_graphql::{dataloader::DataLoader, Context, Enum, Object};

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum TransactType {
    #[graphql(name = "customer_invoice")]
    CustomerInvoice,
    #[graphql(name = "customer_credit")]
    CustomerCredit,
    #[graphql(name = "supplier_invoice")]
    SupplierInvoice,
    #[graphql(name = "supplier_credit")]
    SupplierCredit,
    #[graphql(name = "repack")]
    Repack,
    #[graphql(name = "build")]
    Build,
    #[graphql(name = "receipt")]
    Receipt,
    #[graphql(name = "payment")]
    Payment,
}

impl From<TransactRowType> for TransactType {
    fn from(transact_row_type: TransactRowType) -> TransactType {
        match transact_row_type {
            TransactRowType::CustomerInvoice => TransactType::CustomerInvoice,
            TransactRowType::CustomerCredit => TransactType::CustomerCredit,
            TransactRowType::SupplierInvoice => TransactType::SupplierInvoice,
            TransactRowType::SupplierCredit => TransactType::SupplierCredit,
            TransactRowType::Repack => TransactType::Repack,
            TransactRowType::Build => TransactType::Build,
            TransactRowType::Receipt => TransactType::Receipt,
            TransactRowType::Payment => TransactType::Payment,
        }
    }
}

impl From<TransactType> for TransactRowType {
    fn from(transact_type: TransactType) -> TransactRowType {
        match transact_type {
            TransactType::CustomerInvoice => TransactRowType::CustomerInvoice,
            TransactType::CustomerCredit => TransactRowType::CustomerCredit,
            TransactType::SupplierInvoice => TransactRowType::SupplierInvoice,
            TransactType::SupplierCredit => TransactRowType::SupplierCredit,
            TransactType::Repack => TransactRowType::Repack,
            TransactType::Build => TransactRowType::Build,
            TransactType::Receipt => TransactRowType::Receipt,
            TransactType::Payment => TransactRowType::Payment,
        }
    }
}

#[derive(Clone)]
pub struct Transact {
    pub transact_row: TransactRow,
}

#[Object]
impl Transact {
    pub async fn id(&self) -> String {
        self.transact_row.id.to_string()
    }

    pub async fn name(&self, ctx: &Context<'_>) -> Name {
        let name_loader = ctx.get_loader::<DataLoader<NameLoader>>();

        let name_row: NameRow = name_loader
            .load_one(self.transact_row.name_id.clone())
            .await
            .unwrap_or_else(|_| panic!("Failed to get name for transact {}", self.transact_row.id))
            .ok_or_else(|| panic!("Failed to get name for transact {}", self.transact_row.id))
            .unwrap_or_else(|_| panic!("Failed to get name for transact {}", self.transact_row.id));

        Name { name_row }
    }

    pub async fn invoice_number(&self) -> i32 {
        self.transact_row.invoice_number
    }

    pub async fn type_of(&self) -> TransactType {
        self.transact_row.type_of.clone().into()
    }

    pub async fn transact_lines(&self, ctx: &Context<'_>) -> Vec<TransactLine> {
        let transact_line_repository = ctx.get_repository::<TransactLineRepository>();

        let transact_line_rows: Vec<TransactLineRow> = transact_line_repository
            .find_many_by_transact_id(&self.transact_row.id)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get transact_lines for transact {}",
                    self.transact_row.id
                )
            });

        transact_line_rows
            .into_iter()
            .map(|transact_line_row| TransactLine { transact_line_row })
            .collect()
    }
}
