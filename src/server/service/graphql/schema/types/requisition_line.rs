use crate::{
    database::{
        loader::ItemLoader,
        schema::{ItemRow, RequisitionLineRow},
    },
    server::service::graphql::{schema::types::Item, ContextExt},
};

use async_graphql::{dataloader::DataLoader, Context, InputObject, Object};

#[derive(Clone)]
pub struct RequisitionLine {
    pub requisition_line_row: RequisitionLineRow,
}

#[Object]
impl RequisitionLine {
    pub async fn id(&self) -> &str {
        &self.requisition_line_row.id
    }

    pub async fn item(&self, ctx: &Context<'_>) -> Item {
        let item_loader = ctx.get_loader::<DataLoader<ItemLoader>>();

        let item_row: ItemRow = item_loader
            .load_one(self.requisition_line_row.item_id.clone())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get item for requisition_line {}",
                    self.requisition_line_row.id
                )
            })
            .ok_or_else(|| {
                panic!(
                    "Failed to get item for requisition_line {}",
                    self.requisition_line_row.id
                )
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get item for requisition_line {}",
                    self.requisition_line_row.id
                )
            });

        Item { item_row }
    }

    pub async fn actual_quantity(&self) -> f64 {
        self.requisition_line_row.actual_quantity
    }

    pub async fn suggested_quantity(&self) -> f64 {
        self.requisition_line_row.suggested_quantity
    }
}

#[derive(Clone, InputObject)]
pub struct InputRequisitionLine {
    pub id: String,
    pub item_id: String,
    pub actual_quantity: f64,
    pub suggested_quantity: f64,
}
