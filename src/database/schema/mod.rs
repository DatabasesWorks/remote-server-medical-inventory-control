mod item;
mod item_line;
mod item_property;
mod item_property_join;
mod name;
mod requisition;
mod requisition_line;
mod store;
mod transact;
mod transact_line;
mod user_account;

#[derive(Clone)]
pub enum DatabaseRow {
    Item(ItemRow),
    ItemLine(ItemLineRow),
    Name(NameRow),
    Requisition(RequisitionRow),
    RequisitionLine(RequisitionLineRow),
    Store(StoreRow),
    Transact(TransactRow),
    TransactLine(TransactLineRow),
    UserAccount(UserAccountRow),
    ItemProperty(ItemPropertyRow),
    ItemPropertyJoin(ItemPropertyJoinRow),
}

pub use item::{ItemRow, ItemRowType};
pub use item_line::ItemLineRow;
pub use item_property::ItemPropertyRow;
pub use item_property_join::ItemPropertyJoinRow;
pub use name::NameRow;
pub use requisition::{RequisitionRow, RequisitionRowType};
pub use requisition_line::RequisitionLineRow;
pub use store::StoreRow;
pub use transact::{TransactRow, TransactRowType};
pub use transact_line::{TransactLineRow, TransactLineRowType};
pub use user_account::UserAccountRow;
