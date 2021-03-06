use crate::schema::{
    mutations::{
        CannotDeleteInvoiceWithLines, CannotEditInvoice, DeleteResponse,
        InvoiceDoesNotBelongToCurrentStore, NotAnOutboundShipment,
    },
    types::{DatabaseError, ErrorWrapper, RecordNotFound},
};
use repository::StorageConnectionManager;
use service::invoice::{delete_outbound_shipment, DeleteOutboundShipmentError};

use async_graphql::{Interface, Union};

#[derive(Union)]
pub enum DeleteOutboundShipmentResponse {
    Error(ErrorWrapper<DeleteOutboundShipmentErrorInterface>),
    Response(DeleteResponse),
}

pub fn get_delete_outbound_shipment_response(
    connection_manager: &StorageConnectionManager,
    input: String,
) -> DeleteOutboundShipmentResponse {
    use DeleteOutboundShipmentResponse::*;
    match delete_outbound_shipment(connection_manager, input.into()) {
        Ok(id) => Response(DeleteResponse(id)),
        Err(error) => error.into(),
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum DeleteOutboundShipmentErrorInterface {
    RecordNotFound(RecordNotFound),
    CannotEditInvoice(CannotEditInvoice),
    NotAnOutboundShipment(NotAnOutboundShipment),
    InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore),
    CannotDeleteInvoiceWithLines(CannotDeleteInvoiceWithLines),
    DatabaseError(DatabaseError),
}

impl From<DeleteOutboundShipmentError> for DeleteOutboundShipmentResponse {
    fn from(error: DeleteOutboundShipmentError) -> Self {
        use DeleteOutboundShipmentErrorInterface as OutError;
        let error = match error {
            DeleteOutboundShipmentError::InvoiceDoesNotExist => {
                OutError::RecordNotFound(RecordNotFound {})
            }
            DeleteOutboundShipmentError::CannotEditFinalised => {
                OutError::CannotEditInvoice(CannotEditInvoice {})
            }
            DeleteOutboundShipmentError::NotThisStoreInvoice => {
                OutError::InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore {})
            }
            DeleteOutboundShipmentError::InvoiceLinesExists(lines) => {
                OutError::CannotDeleteInvoiceWithLines(CannotDeleteInvoiceWithLines(lines.into()))
            }
            DeleteOutboundShipmentError::DatabaseError(error) => {
                OutError::DatabaseError(DatabaseError(error))
            }
            DeleteOutboundShipmentError::NotAnOutboundShipment => {
                OutError::NotAnOutboundShipment(NotAnOutboundShipment {})
            }
        };

        DeleteOutboundShipmentResponse::Error(ErrorWrapper { error })
    }
}
