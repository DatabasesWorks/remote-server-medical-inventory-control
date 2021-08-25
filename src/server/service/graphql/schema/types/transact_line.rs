use crate::{
    database::{
        loader::{ItemLineLoader, ItemLoader, TransactLoader},
        schema::{ItemLineRow, ItemRow, TransactLineRow, TransactRow},
    },
    server::service::graphql::{
        schema::types::{Item, ItemLine, Transact},
        ContextExt,
    },
};

use async_graphql::{dataloader::DataLoader, Context, Object};

#[derive(Clone)]
pub struct TransactLine {
    pub transact_line_row: TransactLineRow,
}

#[Object]
impl TransactLine {
    pub async fn id(&self) -> &str {
        &self.transact_line_row.id
    }

    pub async fn transact(&self, ctx: &Context<'_>) -> Transact {
        let transact_loader = ctx.get_loader::<DataLoader<TransactLoader>>();

        let transact_row: TransactRow = transact_loader
            .load_one(self.transact_line_row.transact_id.clone())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get transact for transact_line {}",
                    self.transact_line_row.id
                )
            })
            .ok_or_else(|| {
                panic!(
                    "Failed to get transact for transact_line {}",
                    self.transact_line_row.id
                )
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get transact for transact_line {}",
                    self.transact_line_row.id
                )
            });

        Transact { transact_row }
    }

    pub async fn item(&self, ctx: &Context<'_>) -> Item {
        let item_loader = ctx.get_loader::<DataLoader<ItemLoader>>();

        let item_row: ItemRow = item_loader
            .load_one(self.transact_line_row.item_id.clone())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get item for transact_line {}",
                    self.transact_line_row.id
                )
            })
            .ok_or_else(|| {
                panic!(
                    "Failed to get item for transact_line {}",
                    self.transact_line_row.id
                )
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get item for transact_line {}",
                    self.transact_line_row.id
                )
            });

        Item { item_row }
    }

    pub async fn item_line(&self, ctx: &Context<'_>) -> ItemLine {
        let item_line_loader = ctx.get_loader::<DataLoader<ItemLineLoader>>();

        // Handle optional item_line_id correctly.
        let item_line_id = self.transact_line_row.item_line_id.as_ref().unwrap();

        let item_line_row: ItemLineRow = item_line_loader
            .load_one(item_line_id.to_owned())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get item_line for transact_line {}",
                    self.transact_line_row.id
                )
            })
            .ok_or_else(|| {
                panic!(
                    "Failed to get item_line for transact_line {}",
                    self.transact_line_row.id
                )
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get item_line for transact_line {}",
                    self.transact_line_row.id
                )
            });

        ItemLine { item_line_row }
    }
}
