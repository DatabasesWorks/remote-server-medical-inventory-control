use crate::database::repository::NameQueryRepository;
use crate::server::service::graphql::schema::queries::pagination::{
    PaginationOption, MAX_PAGE_SIZE,
};
use crate::server::service::graphql::{schema::queries::pagination::Pagination, ContextExt};
use async_graphql::*;

#[derive(SimpleObject, PartialEq, Debug)]
#[graphql(name = "Name")]
pub struct NameQuery {
    pub id: String,
    pub name: String,
    pub code: String,
    // Below are from name_store_join
    pub is_customer: bool,
    pub is_supplier: bool,
}

pub struct FirstError(pub i64);

#[Object]
impl FirstError {
    pub async fn description(&self) -> &'static str {
        if self.0 < 1 {
            "First must be at least one"
        } else {
            "First is too large"
        }
    }

    pub async fn first(&self) -> i64 {
        self.0
    }

    pub async fn min_first(&self) -> i64 {
        1
    }

    pub async fn max_first(&self) -> u32 {
        MAX_PAGE_SIZE
    }
}

pub struct OffsetError(pub i64);

#[Object]
impl OffsetError {
    pub async fn description(&self) -> &'static str {
        "Offset must not be negative"
    }

    pub async fn offset(&self) -> i64 {
        self.0
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum Error {
    FirstError(FirstError),
    OffsetError(OffsetError),
}

pub struct NameListError(pub Vec<Error>);

#[Object]
impl NameListError {
    async fn errors(&self) -> &Vec<Error> {
        &self.0
    }
}

#[derive(Union)]
pub enum NameListWithError {
    NameList(NameList),
    NameListError(NameListError),
}

impl NameListWithError {
    pub fn new(pagination: Option<Pagination>) -> NameListWithError {
        match NameListWithError::check_error(&pagination) {
            Some(errors) => NameListWithError::NameListError(NameListError(errors)),
            None => NameListWithError::NameList(NameList { pagination }),
        }
    }

    pub fn check_error(pagination: &impl PaginationOption) -> Option<Vec<Error>> {
        let mut result = Vec::new();
        if pagination.offset() < 0 {
            result.push(Error::OffsetError(OffsetError(pagination.offset())))
        }
        if pagination.first() < 1 || pagination.first() > MAX_PAGE_SIZE.into() {
            result.push(Error::FirstError(FirstError(pagination.first())))
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }
}

pub struct NameList {
    pub pagination: Option<Pagination>,
}

#[Object]
impl NameList {
    async fn total_count(&self, ctx: &Context<'_>) -> i64 {
        let repository = ctx.get_repository::<NameQueryRepository>();
        repository.count().unwrap()
    }

    async fn nodes(&self, ctx: &Context<'_>) -> Vec<NameQuery> {
        let repository = ctx.get_repository::<NameQueryRepository>();
        repository.all(&self.pagination).unwrap()
    }
}
