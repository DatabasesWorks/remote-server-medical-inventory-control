use crate::{
    database::repository::RepositoryError,
    domain::PaginationOption,
    service::{ListError, ListResult},
};

use async_graphql::*;

// M1 speced API is moved to their own files
// Types defined here are prototype types and should be removed before M1 release to avoid confusion (for consumers and devs)
pub mod name;
pub use self::name::*;

pub mod item;
pub use self::item::*;

pub mod stock_line;
pub use self::stock_line::*;

pub mod invoice_query;
pub use self::invoice_query::*;

pub mod sort_filter_types;
pub use self::sort_filter_types::*;

pub mod invoice_line;
pub use self::invoice_line::*;

// Generic Connector

#[derive(SimpleObject)]
#[graphql(concrete(name = "NameConnector", params(NameNode)))]
#[graphql(concrete(name = "ItemConnector", params(ItemNode)))]
#[graphql(concrete(name = "InvoiceConnector", params(InvoiceNode)))]
pub struct Connector<T: OutputType> {
    total_count: u32,
    nodes: Vec<T>,
}

impl<DomainType, GQLType> From<ListResult<DomainType>> for Connector<GQLType>
where
    GQLType: From<DomainType> + OutputType,
{
    fn from(ListResult { count, rows }: ListResult<DomainType>) -> Self {
        Connector {
            total_count: count,
            nodes: rows.into_iter().map(GQLType::from).collect(),
        }
    }
}

/// Generic Pagination Input
#[derive(InputObject)]
pub struct PaginationInput {
    pub first: Option<u32>,
    pub offset: Option<u32>,
}

impl From<PaginationInput> for PaginationOption {
    fn from(PaginationInput { first, offset }: PaginationInput) -> Self {
        PaginationOption {
            limit: first,
            offset,
        }
    }
}

/// Generic Error Wrapper
#[derive(SimpleObject)]
#[graphql(concrete(name = "ConnectorError", params(ConnectorErrorInterface)))]
pub struct ErrorWrapper<T: OutputType> {
    error: T,
}

// Generic Error Interface
#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum ConnectorErrorInterface {
    DBError(DBError),
    PaginationError(PaginationError),
}

impl From<ListError> for ErrorWrapper<ConnectorErrorInterface> {
    fn from(error: ListError) -> Self {
        let error = match error {
            ListError::DBError(RepositoryError) => {
                ConnectorErrorInterface::DBError(DBError(RepositoryError))
            }
            ListError::LimitBelowMin { limit, min } => {
                ConnectorErrorInterface::PaginationError(PaginationError {
                    out_of_range: FirstOutOfRange::Min(min),
                    first: limit,
                })
            }
            ListError::LimitAboveMax { limit, max } => {
                ConnectorErrorInterface::PaginationError(PaginationError {
                    out_of_range: FirstOutOfRange::Max(max),
                    first: limit,
                })
            }
        };

        ErrorWrapper { error }
    }
}

// Generic Errors
pub struct DBError(pub RepositoryError);

#[Object]
impl DBError {
    pub async fn description(&self) -> &'static str {
        "Dabase Error"
    }

    pub async fn full_error(&self) -> String {
        format!("{:#}", self.0)
    }
}

pub enum FirstOutOfRange {
    Max(u32),
    Min(u32),
}
pub struct PaginationError {
    out_of_range: FirstOutOfRange,
    first: u32,
}

#[Object]
impl PaginationError {
    pub async fn description(&self) -> &'static str {
        match &self.out_of_range {
            FirstOutOfRange::Max(_) => "First is too big",
            FirstOutOfRange::Min(_) => "First is too low",
        }
    }

    pub async fn max(&self) -> Option<u32> {
        match &self.out_of_range {
            FirstOutOfRange::Max(max) => Some(max.clone()),
            _ => None,
        }
    }

    pub async fn min(&self) -> Option<u32> {
        match &self.out_of_range {
            FirstOutOfRange::Min(min) => Some(min.clone()),
            _ => None,
        }
    }

    pub async fn first(&self) -> u32 {
        self.first
    }
}
