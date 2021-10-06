use crate::{
    database::{
        loader::{InvoiceLineQueryLoader, InvoiceLineStatsLoader},
        repository::{InvoiceLineQueryJoin, InvoiceLineStats},
    },
    domain::{
        invoice::{Invoice, InvoiceFilter},
        DatetimeFilter, EqualFilter, SimpleStringFilter,
    },
    server::service::graphql::ContextExt,
    service::{ListError, ListResult},
};

use async_graphql::*;
use chrono::{DateTime, Utc};
use dataloader::DataLoader;

use super::{
    Connector, ConnectorErrorInterface, DatetimeFilterInput, EqualFilterInput,
    EqualFilterStringInput, ErrorWrapper, InvoiceLineNode, SimpleStringFilterInput, SortInput,
};

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
#[graphql(remote = "crate::domain::invoice::InvoiceSortField")]
pub enum InvoiceSortFieldInput {
    Type,
    Status,
    EntryDatetime,
    ConfirmDatetime,
    FinalisedDateTime,
}

pub type InvoiceSortInput = SortInput<InvoiceSortFieldInput>;

#[derive(InputObject, Clone)]
pub struct InvoiceFilterInput {
    pub name_id: Option<EqualFilterStringInput>,
    pub store_id: Option<EqualFilterStringInput>,
    pub r#type: Option<EqualFilterInput<InvoiceNodeType>>,
    pub status: Option<EqualFilterInput<InvoiceNodeStatus>>,
    pub comment: Option<SimpleStringFilterInput>,
    pub their_reference: Option<EqualFilterStringInput>,
    pub entry_datetime: Option<DatetimeFilterInput>,
    pub confirm_datetime: Option<DatetimeFilterInput>,
    pub finalised_datetime: Option<DatetimeFilterInput>,
}

impl From<InvoiceFilterInput> for InvoiceFilter {
    fn from(f: InvoiceFilterInput) -> Self {
        InvoiceFilter {
            name_id: f.name_id.map(EqualFilter::from),
            store_id: f.store_id.map(EqualFilter::from),
            r#type: f.r#type.map(EqualFilter::from),
            status: f.status.map(EqualFilter::from),
            comment: f.comment.map(SimpleStringFilter::from),
            their_reference: f.their_reference.map(EqualFilter::from),
            entry_datetime: f.entry_datetime.map(DatetimeFilter::from),
            confirm_datetime: f.confirm_datetime.map(DatetimeFilter::from),
            finalised_datetime: f.finalised_datetime.map(DatetimeFilter::from),
        }
    }
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
#[graphql(remote = "crate::domain::invoice::InvoiceType")]
pub enum InvoiceNodeType {
    CustomerInvoice,
    SupplierInvoice,
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
#[graphql(remote = "crate::domain::invoice::InvoiceStatus")]
pub enum InvoiceNodeStatus {
    Draft,
    Confirmed,
    Finalised,
}

#[derive(SimpleObject, PartialEq, Debug)]
pub struct InvoiceLinePricing {
    /// total for all invoice lines
    total_after_tax: f64,
}

pub struct InvoiceNode {
    invoice: Invoice,
}

#[Object]
impl InvoiceNode {
    pub async fn id(&self) -> &str {
        &self.invoice.id
    }

    pub async fn other_party_name(&self) -> &str {
        &self.invoice.other_party_name
    }

    pub async fn other_party_id(&self) -> &str {
        &self.invoice.other_party_id
    }

    pub async fn r#type(&self) -> InvoiceNodeType {
        self.invoice.r#type.clone().into()
    }

    pub async fn status(&self) -> InvoiceNodeStatus {
        self.invoice.status.clone().into()
    }

    pub async fn invoice_number(&self) -> i32 {
        self.invoice.invoice_number
    }

    pub async fn their_reference(&self) -> &Option<String> {
        &self.invoice.their_reference
    }

    pub async fn comment(&self) -> &Option<String> {
        &self.invoice.comment
    }

    pub async fn entry_datetime(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(self.invoice.entry_datetime, Utc)
    }

    pub async fn confirmed_datetime(&self) -> Option<DateTime<Utc>> {
        self.invoice
            .confirm_datetime
            .map(|v| DateTime::<Utc>::from_utc(v, Utc))
    }

    pub async fn finalised_datetime(&self) -> Option<DateTime<Utc>> {
        self.invoice
            .finalised_datetime
            .map(|v| DateTime::<Utc>::from_utc(v, Utc))
    }

    pub async fn lines(&self) -> InvoiceLines {
        InvoiceLines {
            invoice_id: self.invoice.id.clone(),
        }
    }

    async fn pricing(&self, ctx: &Context<'_>) -> InvoiceLinePricing {
        let loader = ctx.get_loader::<DataLoader<InvoiceLineStatsLoader>>();

        let result: InvoiceLineStats = loader
            .load_one(self.invoice.id.to_string())
            .await
            // TODO report error
            .unwrap()
            .map_or(
                InvoiceLineStats {
                    invoice_id: self.invoice.id.to_string(),
                    total_after_tax: 0.0,
                },
                |v| v,
            );

        InvoiceLinePricing {
            total_after_tax: result.total_after_tax,
        }
    }
}

struct InvoiceLines {
    invoice_id: String,
}

#[Object]
impl InvoiceLines {
    async fn nodes(&self, ctx: &Context<'_>) -> Vec<InvoiceLineNode> {
        let loader = ctx.get_loader::<DataLoader<InvoiceLineQueryLoader>>();

        let lines: Vec<InvoiceLineQueryJoin> = loader
            .load_one(self.invoice_id.to_string())
            .await
            // TODO handle error:
            .unwrap()
            .map_or(Vec::new(), |v| v);

        lines.into_iter().map(InvoiceLineNode::from).collect()
    }
}

#[derive(Union)]
pub enum InvoicesResponse {
    Error(ErrorWrapper<ConnectorErrorInterface>),
    Response(Connector<InvoiceNode>),
}

impl From<Result<ListResult<Invoice>, ListError>> for InvoicesResponse {
    fn from(result: Result<ListResult<Invoice>, ListError>) -> Self {
        match result {
            Ok(response) => InvoicesResponse::Response(response.into()),
            Err(error) => InvoicesResponse::Error(error.into()),
        }
    }
}

impl From<Invoice> for InvoiceNode {
    fn from(invoice: Invoice) -> Self {
        InvoiceNode { invoice }
    }
}
