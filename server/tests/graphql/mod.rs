use std::sync::RwLock;

use actix_web::{test::read_body, web::Data};
use remote_server::{
    database::repository::get_repositories,
    server::{
        data::{auth::AuthData, LoaderRegistry, RepositoryRegistry},
        service::graphql::{config as graphql_config, loader::get_loaders},
    },
    service::token_bucket::TokenBucket,
    util::settings::Settings,
};

use assert_json_diff::assert_json_eq;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

pub mod common;
mod inbound_shipment_delete;
mod inbound_shipment_insert;
mod inbound_shipment_line_delete;
mod inbound_shipment_line_insert;
mod inbound_shipment_line_update;
mod inbound_shipment_update;
mod invoice_query;
mod invoices;
mod items;
mod names;
mod outbound_shipment_delete;
mod outbound_shipment_insert;
mod outbound_shipment_line_delete;
mod outbound_shipment_line_insert;
mod outbound_shipment_line_update;
mod outbound_shipment_update;
mod requisition;

pub async fn get_gql_result<IN, OUT>(settings: &Settings, query: IN) -> OUT
where
    IN: Serialize,
    OUT: DeserializeOwned,
{
    let repositories = get_repositories(settings).await;
    let loaders = get_loaders(settings).await;

    let repository_registry = actix_web::web::Data::new(RepositoryRegistry { repositories });
    let loader_registry = actix_web::web::Data::new(LoaderRegistry { loaders });

    let auth_data = Data::new(AuthData {
        auth_token_secret: settings.auth.token_secret.to_owned(),
        token_bucket: RwLock::new(TokenBucket::new()),
        // TODO: configure ssl
        debug_no_ssl: true,
    });

    let mut app = actix_web::test::init_service(
        actix_web::App::new()
            .data(repository_registry.clone())
            .data(loader_registry.clone())
            .configure(graphql_config(
                repository_registry,
                loader_registry,
                auth_data,
            )),
    )
    .await;

    let res = actix_web::test::TestRequest::post()
        .header("content-type", "application/json")
        .set_json(&query)
        .uri("/graphql")
        .send_request(&mut app)
        .await;

    let body = read_body(res).await;

    let body_as_string = String::from_utf8(body.clone().to_vec()).unwrap();

    match serde_json::from_slice(&body) {
        Ok(result) => result,
        Err(error) => panic!("failed to deserialize: {} {:#?}", body_as_string, error),
    }
}

async fn run_gql_query(
    settings: &Settings,
    query: &str,
    variables: &Option<serde_json::Value>,
) -> serde_json::Value {
    let repositories = get_repositories(settings).await;
    let loaders = get_loaders(settings).await;

    let repository_registry = actix_web::web::Data::new(RepositoryRegistry { repositories });
    let loader_registry = actix_web::web::Data::new(LoaderRegistry { loaders });

    let auth_data = Data::new(AuthData {
        auth_token_secret: settings.auth.token_secret.to_owned(),
        token_bucket: RwLock::new(TokenBucket::new()),
        // TODO: configure ssl
        debug_no_ssl: true,
    });

    let mut app = actix_web::test::init_service(
        actix_web::App::new()
            .data(repository_registry.clone())
            .data(loader_registry.clone())
            .configure(graphql_config(
                repository_registry,
                loader_registry,
                auth_data,
            )),
    )
    .await;

    let mut payload: String;
    if let Some(variables) = variables {
        payload = format!(
            "{{\"query\":\"{}\",\"variables\":{}}}",
            query,
            variables.to_string()
        );
    } else {
        payload = format!("{{\"query\":\"{}\"}}", query);
    }
    payload = payload.replace("\n", "");

    let req = actix_web::test::TestRequest::post()
        .header("content-type", "application/json")
        .set_payload(payload)
        .uri("/graphql")
        .to_request();

    let res = actix_web::test::read_response(&mut app, req).await;
    let body = String::from_utf8(res.to_vec()).expect("Failed to parse response");
    serde_json::from_str::<Value>(&body).expect(body.as_str())
}

async fn assert_gql_not_found(
    settings: &Settings,
    query: &str,
    variables: &Option<serde_json::Value>,
) -> serde_json::Value {
    let actual = run_gql_query(settings, query, variables).await;
    let error_message = actual["data"].to_string();
    assert!(error_message.contains("RecordNotFound"));
    actual
}

fn assert_gql_no_response_error(value: &serde_json::Value) {
    if let Some(errors) = value.get("errors") {
        assert!(false, "{}", errors.to_string());
    }
}

async fn assert_gql_query(
    settings: &Settings,
    query: &str,
    variables: &Option<serde_json::Value>,
    expected: &serde_json::Value,
) -> serde_json::Value {
    let actual = run_gql_query(settings, query, variables).await;
    // println!("{}", serde_json::to_string_pretty(&actual).unwrap());
    assert_gql_no_response_error(&actual);
    let expected_with_data = json!({
        "data": expected,
    });
    assert_json_eq!(&actual, expected_with_data);
    actual
}

use chrono::{DateTime as ChronoDateTime, NaiveDate, Utc};
use graphql_client::GraphQLQuery;
type DateTime = ChronoDateTime<Utc>;
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct InsertInboundShipmentFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct UpdateInboundShipmentFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct DeleteInboundShipmentFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct InsertInboundShipmentLineFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct UpdateInboundShipmentLineFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct InvoiceFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct DeleteInboundShipmentLineFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct UpdateOutboundShipmentLineFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct InsertOutboundShipmentLineFull;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schema/schema.graphql",
    query_path = "graphql_schema/query.graphql",
    response_derives = "Debug,PartialEq,Clone,Serialize",
    variables_derives = "Debug,PartialEq,Clone",
    normalization = "Rust"
)]
pub struct DeleteOutboundShipmentLineFull;
