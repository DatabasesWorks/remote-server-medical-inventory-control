use crate::{
    invoice::{
        check_invoice_exists, check_invoice_is_editable, check_invoice_type,
        validate::InvoiceIsNotEditable, InvoiceDoesNotExist, WrongInvoiceType,
    },
    invoice_line::{
        inbound_shipment_line::check_batch,
        validate::{
            check_line_belongs_to_invoice, check_line_exists, LineDoesNotExist, NotInvoiceLine,
        },
        BatchIsReserved,
    },
};
use domain::{inbound_shipment::DeleteInboundShipmentLine, invoice::InvoiceType};
use repository::{schema::InvoiceLineRow, StorageConnection};

use super::DeleteInboundShipmentLineError;

pub fn validate(
    input: &DeleteInboundShipmentLine,
    connection: &StorageConnection,
) -> Result<InvoiceLineRow, DeleteInboundShipmentLineError> {
    let line = check_line_exists(&input.id, connection)?;

    let invoice = check_invoice_exists(&input.invoice_id, connection)?;
    // check_store(invoice, connection)?; InvoiceDoesNotBelongToCurrentStore
    check_line_belongs_to_invoice(&line, &invoice)?;
    check_invoice_type(&invoice, InvoiceType::InboundShipment)?;
    check_invoice_is_editable(&invoice)?;
    check_batch(&line, connection)?;

    Ok(line)
}

impl From<LineDoesNotExist> for DeleteInboundShipmentLineError {
    fn from(_: LineDoesNotExist) -> Self {
        DeleteInboundShipmentLineError::LineDoesNotExist
    }
}

impl From<WrongInvoiceType> for DeleteInboundShipmentLineError {
    fn from(_: WrongInvoiceType) -> Self {
        DeleteInboundShipmentLineError::NotAnInboundShipment
    }
}

impl From<InvoiceIsNotEditable> for DeleteInboundShipmentLineError {
    fn from(_: InvoiceIsNotEditable) -> Self {
        DeleteInboundShipmentLineError::CannotEditFinalised
    }
}

impl From<NotInvoiceLine> for DeleteInboundShipmentLineError {
    fn from(error: NotInvoiceLine) -> Self {
        DeleteInboundShipmentLineError::NotThisInvoiceLine(error.0)
    }
}

impl From<BatchIsReserved> for DeleteInboundShipmentLineError {
    fn from(_: BatchIsReserved) -> Self {
        DeleteInboundShipmentLineError::BatchIsReserved
    }
}

impl From<InvoiceDoesNotExist> for DeleteInboundShipmentLineError {
    fn from(_: InvoiceDoesNotExist) -> Self {
        DeleteInboundShipmentLineError::InvoiceDoesNotExist
    }
}
