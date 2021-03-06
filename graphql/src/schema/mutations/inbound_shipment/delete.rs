use async_graphql::*;

use crate::schema::{
    mutations::{
        CannotDeleteInvoiceWithLines, CannotEditInvoice, DeleteResponse,
        InvoiceDoesNotBelongToCurrentStore, NotAnInboundShipment,
    },
    types::{DatabaseError, ErrorWrapper, RecordNotFound},
};
use domain::inbound_shipment::DeleteInboundShipment;
use repository::StorageConnectionManager;
use service::invoice::{delete_inbound_shipment, DeleteInboundShipmentError};

#[derive(InputObject)]
pub struct DeleteInboundShipmentInput {
    pub id: String,
}

#[derive(Union)]
pub enum DeleteInboundShipmentResponse {
    Error(ErrorWrapper<DeleteInboundShipmentErrorInterface>),
    Response(DeleteResponse),
}

pub fn get_delete_inbound_shipment_response(
    connection_manager: &StorageConnectionManager,
    input: DeleteInboundShipmentInput,
) -> DeleteInboundShipmentResponse {
    use DeleteInboundShipmentResponse::*;
    match delete_inbound_shipment(connection_manager, input.into()) {
        Ok(id) => Response(DeleteResponse(id)),
        Err(error) => error.into(),
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum DeleteInboundShipmentErrorInterface {
    DatabaseError(DatabaseError),
    RecordNotFound(RecordNotFound),
    CannotEditInvoice(CannotEditInvoice),
    NotAnInboundShipment(NotAnInboundShipment),
    InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore),
    CannotDeleteInvoiceWithLines(CannotDeleteInvoiceWithLines),
}

impl From<DeleteInboundShipmentInput> for DeleteInboundShipment {
    fn from(input: DeleteInboundShipmentInput) -> Self {
        DeleteInboundShipment { id: input.id }
    }
}

impl From<DeleteInboundShipmentError> for DeleteInboundShipmentResponse {
    fn from(error: DeleteInboundShipmentError) -> Self {
        use DeleteInboundShipmentErrorInterface as OutError;
        let error = match error {
            DeleteInboundShipmentError::InvoiceDoesNotExist => {
                OutError::RecordNotFound(RecordNotFound {})
            }
            DeleteInboundShipmentError::DatabaseError(error) => {
                OutError::DatabaseError(DatabaseError(error))
            }

            DeleteInboundShipmentError::NotAnInboundShipment => {
                OutError::NotAnInboundShipment(NotAnInboundShipment {})
            }
            DeleteInboundShipmentError::NotThisStoreInvoice => {
                OutError::InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore {})
            }
            DeleteInboundShipmentError::CannotEditFinalised => {
                OutError::CannotEditInvoice(CannotEditInvoice {})
            }
            DeleteInboundShipmentError::InvoiceLinesExists(lines) => {
                OutError::CannotDeleteInvoiceWithLines(CannotDeleteInvoiceWithLines(lines.into()))
            }
        };

        DeleteInboundShipmentResponse::Error(ErrorWrapper { error })
    }
}
