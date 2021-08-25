use crate::{
    database::{
        loader::{NameLoader, StoreLoader},
        repository::RequisitionLineRepository,
        schema::{NameRow, RequisitionLineRow, RequisitionRow, RequisitionRowType, StoreRow},
    },
    server::service::graphql::{
        schema::types::{Name, RequisitionLine, Store},
        ContextExt,
    },
};

use async_graphql::{dataloader::DataLoader, Context, Enum, Object};

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum RequisitionType {
    #[graphql(name = "imprest")]
    Imprest,
    #[graphql(name = "stock_history")]
    StockHistory,
    #[graphql(name = "request")]
    Request,
    #[graphql(name = "response")]
    Response,
    #[graphql(name = "supply")]
    Supply,
    #[graphql(name = "report")]
    Report,
}

impl From<RequisitionRowType> for RequisitionType {
    fn from(requisition_row_type: RequisitionRowType) -> RequisitionType {
        match requisition_row_type {
            RequisitionRowType::Imprest => RequisitionType::Imprest,
            RequisitionRowType::StockHistory => RequisitionType::StockHistory,
            RequisitionRowType::Request => RequisitionType::Request,
            RequisitionRowType::Response => RequisitionType::Response,
            RequisitionRowType::Supply => RequisitionType::Supply,
            RequisitionRowType::Report => RequisitionType::Report,
        }
    }
}

impl From<RequisitionType> for RequisitionRowType {
    fn from(requisition_type: RequisitionType) -> RequisitionRowType {
        match requisition_type {
            RequisitionType::Imprest => RequisitionRowType::Imprest,
            RequisitionType::StockHistory => RequisitionRowType::StockHistory,
            RequisitionType::Request => RequisitionRowType::Request,
            RequisitionType::Response => RequisitionRowType::Response,
            RequisitionType::Supply => RequisitionRowType::Supply,
            RequisitionType::Report => RequisitionRowType::Report,
        }
    }
}

#[derive(Clone)]
pub struct Requisition {
    pub requisition_row: RequisitionRow,
}

#[Object]
impl Requisition {
    pub async fn id(&self) -> &str {
        &self.requisition_row.id
    }

    pub async fn name(&self, ctx: &Context<'_>) -> Name {
        let name_loader = ctx.get_loader::<DataLoader<NameLoader>>();

        let name_row: NameRow = name_loader
            .load_one(self.requisition_row.name_id.clone())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get name for requisition {}",
                    self.requisition_row.id
                )
            })
            .ok_or_else(|| {
                panic!(
                    "Failed to get name for requisition {}",
                    self.requisition_row.id
                )
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get name for requisition {}",
                    self.requisition_row.id
                )
            });

        Name { name_row }
    }

    pub async fn store(&self, ctx: &Context<'_>) -> Store {
        let store_loader = ctx.get_loader::<DataLoader<StoreLoader>>();

        let store_row: StoreRow = store_loader
            .load_one(self.requisition_row.store_id.clone())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get store for requisition {}",
                    self.requisition_row.id
                )
            })
            .ok_or_else(|| {
                panic!(
                    "Failed to get store for requisition {}",
                    self.requisition_row.id
                )
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get store for requisition {}",
                    self.requisition_row.id
                )
            });

        Store { store_row }
    }

    pub async fn type_of(&self) -> RequisitionType {
        self.requisition_row.type_of.clone().into()
    }

    pub async fn requisition_lines(&self, ctx: &Context<'_>) -> Vec<RequisitionLine> {
        let requisition_line_repository = ctx.get_repository::<RequisitionLineRepository>();

        let requisition_line_rows: Vec<RequisitionLineRow> = requisition_line_repository
            .find_many_by_requisition_id(&self.requisition_row.id)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get lines for requisition {}",
                    self.requisition_row.id
                )
            });

        requisition_line_rows
            .into_iter()
            .map(|requisition_line_row| RequisitionLine {
                requisition_line_row,
            })
            .collect()
    }
}
