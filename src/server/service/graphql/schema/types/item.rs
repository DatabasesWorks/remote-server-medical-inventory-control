use crate::database::schema::{ItemRow, ItemRowType};

use async_graphql::{Enum, Object};

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum ItemType {
    #[graphql(name = "general")]
    General,
    #[graphql(name = "service")]
    Service,
    #[graphql(name = "cross_reference")]
    CrossReference,
}

impl From<ItemRowType> for ItemType {
    fn from(item_type: ItemRowType) -> ItemType {
        match item_type {
            ItemRowType::General => ItemType::General,
            ItemRowType::Service => ItemType::Service,
            ItemRowType::CrossReference => ItemType::CrossReference,
        }
    }
}

impl From<ItemType> for ItemRowType {
    fn from(item_type: ItemType) -> ItemRowType {
        match item_type {
            ItemType::General => ItemRowType::General,
            ItemType::Service => ItemRowType::Service,
            ItemType::CrossReference => ItemRowType::CrossReference,
        }
    }
}

#[derive(Clone)]
pub struct Item {
    pub item_row: ItemRow,
}

#[Object]
impl Item {
    pub async fn id(&self) -> &str {
        &self.item_row.id
    }

    pub async fn name(&self) -> &str {
        &self.item_row.name
    }

    pub async fn type_of(&self) -> ItemType {
        self.item_row.type_of.clone().into()
    }
}
