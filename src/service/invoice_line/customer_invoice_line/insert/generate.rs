use crate::{
    database::schema::{InvoiceLineRow, InvoiceRow, InvoiceRowStatus, ItemRow, StockLineRow},
    domain::customer_invoice::InsertCustomerInvoiceLine,
    service::u32_to_i32,
};

use super::InsertCustomerInvoiceLineError;

pub fn generate(
    input: InsertCustomerInvoiceLine,
    item_row: ItemRow,
    batch: StockLineRow,
    invoice: InvoiceRow,
) -> Result<(InvoiceLineRow, StockLineRow), InsertCustomerInvoiceLineError> {
    let adjust_total_number_of_packs = invoice.status == InvoiceRowStatus::Confirmed;

    let update_batch = generate_batch_update(&input, batch.clone(), adjust_total_number_of_packs);
    let new_line = generate_line(input, item_row, batch);

    Ok((new_line, update_batch))
}

fn generate_batch_update(
    input: &InsertCustomerInvoiceLine,
    batch: StockLineRow,
    adjust_total_number_of_packs: bool,
) -> StockLineRow {
    let mut update_batch = batch;

    let reduction = u32_to_i32(input.number_of_packs);

    update_batch.available_number_of_packs = update_batch.available_number_of_packs - reduction;
    if adjust_total_number_of_packs {
        update_batch.total_number_of_packs = update_batch.total_number_of_packs - reduction;
    }

    update_batch
}

fn generate_line(
    InsertCustomerInvoiceLine {
        id,
        invoice_id,
        item_id,
        stock_line_id,
        number_of_packs,
    }: InsertCustomerInvoiceLine,
    ItemRow {
        name: item_name,
        code: item_code,
        ..
    }: ItemRow,
    StockLineRow {
        sell_price_per_pack,
        cost_price_per_pack,
        pack_size,
        batch,
        expiry_date,
        ..
    }: StockLineRow,
) -> InvoiceLineRow {
    let total_after_tax = sell_price_per_pack * pack_size as f64 * number_of_packs as f64;

    InvoiceLineRow {
        id,
        invoice_id,
        item_id,
        pack_size,
        batch,
        expiry_date,
        sell_price_per_pack,
        cost_price_per_pack,
        number_of_packs: u32_to_i32(number_of_packs),
        item_name,
        item_code,
        stock_line_id: Some(stock_line_id),
        total_after_tax,
    }
}