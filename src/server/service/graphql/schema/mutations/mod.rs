pub mod customer_invoice;
mod error;
pub mod supplier_invoice;

use super::types::{Connector, InvoiceLineNode, InvoiceResponse};
use crate::{
    database::repository::StorageConnectionManager,
    server::service::graphql::ContextExt,
    service::{
        invoice::{
            delete_customer_invoice, delete_supplier_invoice, get_invoice, insert_customer_invoice,
            insert_supplier_invoice, update_customer_invoice, update_supplier_invoice,
        },
        invoice_line::{
            delete_customer_invoice_line, delete_supplier_invoice_line, get_invoice_line,
            insert_customer_invoice_line, insert_supplier_invoice_line,
            update_customer_invoice_line, update_supplier_invoice_line,
        },
    },
};
use async_graphql::*;
use customer_invoice::*;
use supplier_invoice::*;

#[derive(SimpleObject)]
#[graphql(concrete(
    name = "InsertCustomerInvoiceResponseWithId",
    params(InsertCustomerInvoiceResponse)
))]
#[graphql(concrete(
    name = "InsertSupplierInvoiceResponseWithId",
    params(InsertSupplierInvoiceResponse)
))]
#[graphql(concrete(
    name = "UpdateCustomerInvoiceResponseWithId",
    params(UpdateCustomerInvoiceResponse)
))]
#[graphql(concrete(
    name = "UpdateSupplierInvoiceResponseWithId",
    params(UpdateSupplierInvoiceResponse)
))]
#[graphql(concrete(
    name = "DeleteCustomerInvoiceResponseWithId",
    params(DeleteCustomerInvoiceResponse)
))]
#[graphql(concrete(
    name = "DeleteSupplierInvoiceResponseWithId",
    params(DeleteSupplierInvoiceResponse)
))]
#[graphql(concrete(
    name = "InsertCustomerInvoiceLineResponseWithId",
    params(InsertCustomerInvoiceLineResponse)
))]
#[graphql(concrete(
    name = "InsertSupplierInvoiceLineResponseWithId",
    params(InsertSupplierInvoiceLineResponse)
))]
#[graphql(concrete(
    name = "UpdateCustomerInvoiceLineResponseWithId",
    params(UpdateCustomerInvoiceLineResponse)
))]
#[graphql(concrete(
    name = "UpdateSupplierInvoiceLineResponseWithId",
    params(UpdateSupplierInvoiceLineResponse)
))]
#[graphql(concrete(
    name = "DeleteCustomerInvoiceLineResponseWithId",
    params(DeleteCustomerInvoiceLineResponse)
))]
#[graphql(concrete(
    name = "DeleteSupplierInvoiceLineResponseWithId",
    params(DeleteSupplierInvoiceLineResponse)
))]
pub struct MutationWithId<T: OutputType> {
    pub id: String,
    pub result: T,
}

#[derive(SimpleObject)]
pub struct AllMutations {
    insertCustomerInvoice: Option<Vec<MutationWithId<InsertCustomerInvoiceResponse>>>,
    insertSupplierInvoice: Option<Vec<MutationWithId<InsertSupplierInvoiceResponse>>>,

    insertCustomerInvoiceLine: Option<Vec<MutationWithId<InsertCustomerInvoiceLineResponse>>>,
    insertSupplierInvoiceLine: Option<Vec<MutationWithId<InsertSupplierInvoiceLineResponse>>>,

    updateCustomerInvoiceLine: Option<Vec<MutationWithId<UpdateCustomerInvoiceLineResponse>>>,
    updateSupplierInvoiceLine: Option<Vec<MutationWithId<UpdateSupplierInvoiceLineResponse>>>,

    deleteCustomerInvoiceLine: Option<Vec<MutationWithId<DeleteCustomerInvoiceLineResponse>>>,
    deleteSupplierInvoiceLine: Option<Vec<MutationWithId<DeleteSupplierInvoiceLineResponse>>>,

    updateCustomerInvoice: Option<Vec<MutationWithId<UpdateCustomerInvoiceResponse>>>,
    updateSupplierInvoice: Option<Vec<MutationWithId<UpdateSupplierInvoiceResponse>>>,

    deleteCustomerInvoice: Option<Vec<MutationWithId<DeleteCustomerInvoiceResponse>>>,
    deleteSupplierInvoice: Option<Vec<MutationWithId<DeleteSupplierInvoiceResponse>>>,
}

pub struct Mutations;

#[Object]
impl Mutations {
    async fn all(
        &self,
        ctx: &Context<'_>,
        insertCustomerInvoice: Option<Vec<InsertCustomerInvoiceInput>>,
        insertSupplierInvoice: Option<Vec<InsertSupplierInvoiceInput>>,

        insertSupplierInvoiceLine: Option<Vec<InsertSupplierInvoiceLineInput>>,
        insertCustomerInvoiceLine: Option<Vec<InsertCustomerInvoiceLineInput>>,

        updateCustomerInvoiceLine: Option<Vec<UpdateCustomerInvoiceLineInput>>,
        updateSupplierInvoiceLine: Option<Vec<UpdateSupplierInvoiceLineInput>>,

        deleteCustomerInvoiceLine: Option<Vec<DeleteCustomerInvoiceLineInput>>,
        deleteSupplierInvoiceLine: Option<Vec<DeleteSupplierInvoiceLineInput>>,

        updateCustomerInvoice: Option<Vec<InsertCustomerInvoiceInput>>,
        updateSupplierInvoice: Option<Vec<InsertSupplierInvoiceInput>>,

        deleteCustomerInvoice: Option<Vec<String>>,
        deleteSupplierInvoice: Option<Vec<DeleteSupplierInvoiceInput>>,
    ) -> AllMutations {
        todo!();
    }

    async fn insert_customer_invoice(
        &self,
        ctx: &Context<'_>,
        input: InsertCustomerInvoiceInput,
    ) -> InsertCustomerInvoiceResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();
        match insert_customer_invoice(connection_manager, input.into()) {
            Ok(id) => get_invoice(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn update_customer_invoice(
        &self,
        ctx: &Context<'_>,
        input: UpdateCustomerInvoiceInput,
    ) -> UpdateCustomerInvoiceResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();
        match update_customer_invoice(connection_manager, input.into()) {
            Ok(id) => get_invoice(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn delete_customer_invoice(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> DeleteCustomerInvoiceResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();
        delete_customer_invoice(connection_manager, id).into()
    }

    async fn insert_customer_invoice_line(
        &self,
        ctx: &Context<'_>,
        input: InsertCustomerInvoiceLineInput,
    ) -> InsertCustomerInvoiceLineResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        match insert_customer_invoice_line(connection_manager, input.into()) {
            Ok(id) => get_invoice_line(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn update_customer_invoice_line(
        &self,
        ctx: &Context<'_>,
        input: UpdateCustomerInvoiceLineInput,
    ) -> UpdateCustomerInvoiceLineResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        match update_customer_invoice_line(connection_manager, input.into()) {
            Ok(id) => get_invoice_line(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn delete_customer_invoice_line(
        &self,
        ctx: &Context<'_>,
        input: DeleteCustomerInvoiceLineInput,
    ) -> DeleteCustomerInvoiceLineResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        delete_customer_invoice_line(connection_manager, input.into()).into()
    }

    async fn insert_supplier_invoice(
        &self,
        ctx: &Context<'_>,
        input: InsertSupplierInvoiceInput,
    ) -> InsertSupplierInvoiceResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        match insert_supplier_invoice(connection_manager, input.into()) {
            Ok(id) => get_invoice(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn update_supplier_invoice(
        &self,
        ctx: &Context<'_>,
        input: UpdateSupplierInvoiceInput,
    ) -> UpdateSupplierInvoiceResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        match update_supplier_invoice(connection_manager, input.into()) {
            Ok(id) => get_invoice(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn delete_supplier_invoice(
        &self,
        ctx: &Context<'_>,
        input: DeleteSupplierInvoiceInput,
    ) -> DeleteSupplierInvoiceResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        delete_supplier_invoice(connection_manager, input.into()).into()
    }

    async fn insert_supplier_invoice_line(
        &self,
        ctx: &Context<'_>,
        input: InsertSupplierInvoiceLineInput,
    ) -> InsertSupplierInvoiceLineResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        match insert_supplier_invoice_line(connection_manager, input.into()) {
            Ok(id) => get_invoice_line(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn update_supplier_invoice_line(
        &self,
        ctx: &Context<'_>,
        input: UpdateSupplierInvoiceLineInput,
    ) -> UpdateSupplierInvoiceLineResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        match update_supplier_invoice_line(connection_manager, input.into()) {
            Ok(id) => get_invoice_line(connection_manager, id).into(),
            Err(error) => error.into(),
        }
    }

    async fn delete_supplier_invoice_line(
        &self,
        ctx: &Context<'_>,
        input: DeleteSupplierInvoiceLineInput,
    ) -> DeleteSupplierInvoiceLineResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        delete_supplier_invoice_line(connection_manager, input.into()).into()
    }
}

// Common Mutation Errors
#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum ForeignKey {
    OtherPartyId,
    ItemId,
    InvoiceId,
    StockLineId,
}

pub struct ForeignKeyError(ForeignKey);
#[Object]
impl ForeignKeyError {
    pub async fn description(&self) -> &'static str {
        "FK record doesn't exist"
    }

    pub async fn key(&self) -> ForeignKey {
        self.0
    }
}

pub struct RecordDoesNotExist;
#[Object]
impl RecordDoesNotExist {
    pub async fn description(&self) -> &'static str {
        "Record does not exist"
    }
}

pub struct RecordAlreadyExist;
#[Object]
impl RecordAlreadyExist {
    pub async fn description(&self) -> &'static str {
        "Record already exists"
    }
}

pub struct CannotEditFinalisedInvoice;
#[Object]
impl CannotEditFinalisedInvoice {
    pub async fn description(&self) -> &'static str {
        "Cannot edit finalised invoice"
    }
}

pub struct NotASupplierInvoice;
#[Object]
impl NotASupplierInvoice {
    pub async fn description(&self) -> &'static str {
        "Invoice is not Supplier Invoice"
    }
}

pub struct NotACustomerInvoice;
#[Object]
impl NotACustomerInvoice {
    pub async fn description(&self) -> &'static str {
        "Invoice is not Customer Invoice"
    }
}

pub struct CannotDeleteInvoiceWithLines(pub Connector<InvoiceLineNode>);
#[Object]
impl CannotDeleteInvoiceWithLines {
    pub async fn description(&self) -> &'static str {
        "Cannot delete invoice with existing lines"
    }

    pub async fn lines(&self) -> &Connector<InvoiceLineNode> {
        &self.0
    }
}

pub struct InvoiceDoesNotBelongToCurrentStore;
#[Object]
impl InvoiceDoesNotBelongToCurrentStore {
    pub async fn description(&self) -> &'static str {
        "Invoice does not belong to current store"
    }
}

pub struct CannotChangeInvoiceBackToDraft;
#[Object]
impl CannotChangeInvoiceBackToDraft {
    pub async fn description(&self) -> &'static str {
        "Cannot change invoice back to draft"
    }
}

pub struct DeleteResponse(String);
#[Object]
impl DeleteResponse {
    pub async fn id(&self) -> &str {
        &self.0
    }
}

pub struct InvoiceLineBelongsToAnotherInvoice(String);
#[Object]
impl InvoiceLineBelongsToAnotherInvoice {
    pub async fn description(&self) -> &'static str {
        "Invoice line belongs to another invoice"
    }

    pub async fn invoice(&self, ctx: &Context<'_>) -> InvoiceResponse {
        let connection_manager = ctx.get_repository::<StorageConnectionManager>();

        get_invoice(connection_manager, self.0.clone()).into()
    }
}
