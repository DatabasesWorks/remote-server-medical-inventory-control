use crate::current_store_id;
use repository::{
    schema::{InvoiceLineRow, StockLineRow},
    RepositoryError, StorageConnection,
};
use util::uuid::uuid;

pub fn generate_batch(
    InvoiceLineRow {
        stock_line_id,
        item_id,
        pack_size,
        batch,
        expiry_date,
        sell_price_per_pack,
        cost_price_per_pack,
        number_of_packs,
        location_id,
        note,
        ..
    }: InvoiceLineRow,
    keep_existing_batch: bool,
    connection: &StorageConnection,
) -> Result<StockLineRow, RepositoryError> {
    // Generate new id if requested via parameter or if stock_line_id is not already set on line
    let stock_line_id = match (stock_line_id, keep_existing_batch) {
        (Some(stock_line_id), true) => stock_line_id,
        _ => uuid(),
    };

    let result = StockLineRow {
        id: stock_line_id,
        item_id,
        store_id: current_store_id(connection)?,
        location_id,
        batch,
        pack_size,
        cost_price_per_pack,
        sell_price_per_pack,
        available_number_of_packs: number_of_packs,
        total_number_of_packs: number_of_packs,
        expiry_date,
        on_hold: false,
        note,
    };

    Ok(result)
}
