use domain::{inbound_shipment::UpdateInboundShipmentLine, invoice::InvoiceType};
use repository::{
    schema::{InvoiceLineRow, InvoiceRow, InvoiceRowStatus, ItemRow},
    InvoiceLineRowRepository, InvoiceRepository, ItemRepository, LocationRowRepository,
    RepositoryError, StockLineRowRepository, StorageConnection,
};

use super::UpdateInboundShipmentLineError;

fn check_line_exists(
    id: &str,
    connection: &StorageConnection,
) -> Result<Option<InvoiceLineRow>, RepositoryError> {
    let result = InvoiceLineRowRepository::new(connection).find_one_by_id(id);
    match result {
        Ok(line) => Ok(Some(line)),
        Err(RepositoryError::NotFound) => Ok(None),
        Err(error) => Err(error),
    }
}

fn pack_size_below_zero(pack_size_option: Option<u32>) -> bool {
    if let Some(pack_size) = pack_size_option {
        if pack_size < 1 {
            return true;
        }
    }
    false
}

fn number_of_packs_below_one(number_of_packs_option: Option<u32>) -> bool {
    if let Some(number_of_packs) = number_of_packs_option {
        if number_of_packs < 1 {
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn check_item(
    item_id: &str,
    connection: &StorageConnection,
) -> Result<Option<ItemRow>, RepositoryError> {
    let item_result = ItemRepository::new(connection).find_one_by_id(item_id);

    match item_result {
        Ok(item) => Ok(Some(item)),
        Err(RepositoryError::NotFound) => Ok(None),
        Err(error) => Err(error),
    }
}

fn check_invoice_exists(
    id: &str,
    connection: &StorageConnection,
) -> Result<Option<InvoiceRow>, RepositoryError> {
    let result = InvoiceRepository::new(connection).find_one_by_id(id);

    match result {
        Ok(invoice_row) => Ok(Some(invoice_row)),
        Err(RepositoryError::NotFound) => Ok(None),
        Err(error) => Err(error),
    }
}

fn check_line_belongs_to_invoice(line: &InvoiceLineRow, invoice: &InvoiceRow) -> bool {
    line.invoice_id == invoice.id
}

fn check_invoice_type(invoice: &InvoiceRow, r#type: InvoiceType) -> bool {
    invoice.r#type == r#type.into()
}

fn check_invoice_is_not_finalised(invoice: &InvoiceRow) -> bool {
    invoice.status != InvoiceRowStatus::Finalised
}

fn check_batch_stock_reserved(
    line: &InvoiceLineRow,
    connection: &StorageConnection,
) -> Result<bool, RepositoryError> {
    if let Some(batch_id) = &line.stock_line_id {
        let batch = StockLineRowRepository::new(connection).find_one_by_id(batch_id)?;
        return Ok(line.number_of_packs == batch.available_number_of_packs);
    }

    return Ok(true);
}

fn check_location_exists(
    location_id: &Option<String>,
    connection: &StorageConnection,
) -> Result<bool, RepositoryError> {
    match location_id {
        Some(location_id) => {
            let location = LocationRowRepository::new(connection).find_one_by_id(&location_id)?;
            match location {
                Some(_) => Ok(true),
                None => Ok(false),
            }
        }
        None => Ok(true),
    }
}

pub fn validate(
    input: &UpdateInboundShipmentLine,
    connection: &StorageConnection,
) -> Result<(InvoiceLineRow, Option<ItemRow>, InvoiceRow), UpdateInboundShipmentLineError> {
    let line = match check_line_exists(&input.id, connection)? {
        Some(line) => line,
        None => return Err(UpdateInboundShipmentLineError::LineDoesNotExist),
    };
    if pack_size_below_zero(input.pack_size.clone()) {
        return Err(UpdateInboundShipmentLineError::PackSizeBelowOne);
    }
    if number_of_packs_below_one(input.number_of_packs.clone()) {
        return Err(UpdateInboundShipmentLineError::NumberOfPacksBelowOne);
    }
    let item = if let Some(item_id) = &input.item_id {
        match check_item(item_id, connection)? {
            Some(item) => Some(item),
            None => return Err(UpdateInboundShipmentLineError::ItemNotFound),
        }
    } else {
        None
    };
    let invoice = match check_invoice_exists(&input.invoice_id, connection)? {
        Some(row) => row,
        None => return Err(UpdateInboundShipmentLineError::InvoiceDoesNotExist),
    };
    if !check_line_belongs_to_invoice(&line, &invoice) {
        return Err(UpdateInboundShipmentLineError::NotThisInvoiceLine(
            line.invoice_id,
        ));
    }
    if !check_invoice_type(&invoice, InvoiceType::InboundShipment) {
        return Err(UpdateInboundShipmentLineError::NotAnInboundShipment);
    }
    if !check_invoice_is_not_finalised(&invoice) {
        return Err(UpdateInboundShipmentLineError::CannotEditFinalised);
    }
    if !check_batch_stock_reserved(&line, connection)? {
        return Err(UpdateInboundShipmentLineError::BatchIsReserved);
    }
    if !check_location_exists(&input.location_id, connection)? {
        return Err(UpdateInboundShipmentLineError::LocationDoesNotExists);
    }

    // TODO: InvoiceDoesNotBelongToCurrentStore
    // TODO: StockLineDoesNotBelongToCurrentStore
    // TODO: LocationDoesNotBelongToCurrentStore

    Ok((line, item, invoice))
}
