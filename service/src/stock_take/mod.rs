use repository::StockTake;

use crate::service_provider::ServiceContext;

use self::{
    delete::{delete_stock_take, DeleteStockTakeError, DeleteStockTakeInput},
    insert::{insert_stock_take, InsertStockTakeError, InsertStockTakeInput},
};

pub mod delete;
pub mod insert;
pub mod query;
pub mod update;
pub mod validate;

#[cfg(test)]
mod tests;

pub trait StockTakeServiceTrait: Sync + Send {
    fn insert_stock_take(
        &self,
        ctx: &ServiceContext,
        input: InsertStockTakeInput,
    ) -> Result<StockTake, InsertStockTakeError> {
        insert_stock_take(ctx, input)
    }

    fn delete_stock_take(
        &self,
        ctx: &ServiceContext,
        input: DeleteStockTakeInput,
    ) -> Result<String, DeleteStockTakeError> {
        delete_stock_take(ctx, input)
    }
}

pub struct StockTakeService {}
impl StockTakeServiceTrait for StockTakeService {}
