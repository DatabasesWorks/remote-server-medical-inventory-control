use crate::{
    database::{
        loader::{ItemLoader, StoreLoader},
        schema::{ItemLineRow, ItemRow, StoreRow},
    },
    server::service::graphql::{
        schema::types::{Item, Store},
        ContextExt,
    },
};

use async_graphql::{dataloader::DataLoader, Context, Object};

#[derive(Clone)]
pub struct ItemLine {
    pub item_line_row: ItemLineRow,
}

#[Object]
impl ItemLine {
    pub async fn id(&self) -> &str {
        &self.item_line_row.id
    }

    pub async fn item(&self, ctx: &Context<'_>) -> Item {
        let item_loader = ctx.get_loader::<DataLoader<ItemLoader>>();

        let item_row: ItemRow = item_loader
            .load_one(self.item_line_row.item_id.clone())
            .await
            .unwrap_or_else(|_| {
                panic!("Failed to get item for item_line {}", self.item_line_row.id)
            })
            .ok_or_else(|| panic!("Failed to get item for item_line {}", self.item_line_row.id))
            .unwrap_or_else(|_| {
                panic!("Failed to get item for item_line {}", self.item_line_row.id)
            });

        Item { item_row }
    }

    pub async fn store(&self, ctx: &Context<'_>) -> Store {
        let store_loader = ctx.get_loader::<DataLoader<StoreLoader>>();

        let store_row: StoreRow = store_loader
            .load_one(self.item_line_row.store_id.clone())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get store for item_line {}",
                    self.item_line_row.id
                )
            })
            .ok_or_else(|| {
                panic!(
                    "Failed to get store for item_line {}",
                    self.item_line_row.id
                )
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get store for item_line {}",
                    self.item_line_row.id
                )
            });

        Store { store_row }
    }

    pub async fn batch(&self) -> &str {
        &self.item_line_row.batch
    }

    pub async fn quantity(&self) -> f64 {
        self.item_line_row.quantity
    }
}
