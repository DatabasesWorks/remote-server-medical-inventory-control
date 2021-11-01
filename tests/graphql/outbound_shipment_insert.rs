#![allow(where_clauses_object_safety)]

mod graphql {
    use crate::graphql::assert_gql_query;
    use remote_server::{
        database::{
            mock::{mock_name_store_joins, mock_names, mock_stores},
            repository::{
                get_repositories, InvoiceRepository, NameRepository, NameStoreJoinRepository,
                StorageConnectionManager, StoreRepository,
            },
            schema::{NameRow, NameStoreJoinRow, StoreRow},
        },
        util::test_db,
    };
    use serde_json::json;

    #[actix_rt::test]
    async fn test_graphql_outbound_shipment_insert() {
        let settings = test_db::get_test_settings("omsupply-database-gql-outbound_shipment_insert");
        test_db::setup(&settings.database).await;
        let repositories = get_repositories(&settings).await;
        let connection_manager = repositories.get::<StorageConnectionManager>().unwrap();
        let connection = connection_manager.connection().unwrap();

        // setup
        let name_repository = NameRepository::new(&connection);
        let store_repository = StoreRepository::new(&connection);
        let name_store_repository = NameStoreJoinRepository::new(&connection);
        let invoice_repository = InvoiceRepository::new(&connection);
        let mock_names: Vec<NameRow> = mock_names();
        let mock_stores: Vec<StoreRow> = mock_stores();
        let mock_name_store_joins: Vec<NameStoreJoinRow> = mock_name_store_joins();
        for name in &mock_names {
            name_repository.insert_one(&name).await.unwrap();
        }
        for store in mock_stores {
            store_repository.insert_one(&store).await.unwrap();
        }
        for name_store_join in &mock_name_store_joins {
            name_store_repository.upsert_one(name_store_join).unwrap();
        }

        let other_party_supplier = &mock_names[2];
        let other_party_customer = &mock_names[0];

        let query = r#"mutation InsertOutboundShipment($input: InsertOutboundShipmentInput!) {
            insertOutboundShipment(input: $input) {
                ... on InsertOutboundShipmentError {
                  error {
                    __typename
                  }
                }
                ... on NodeError {
                  error {
                    __typename
                  }
                }
                ... on InvoiceNode {
                    id
                    otherPartyId
                    type
                    comment
                }
            }
        }"#;

        // OtherPartyNotACustomerError
        let variables = Some(json!({
          "input": {
            "id": "ci_insert_1",
            "otherPartyId": other_party_supplier.id,
          }
        }));
        let expected = json!({
            "insertOutboundShipment": {
              "error": {
                "__typename": "OtherPartyNotACustomerError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // ForeignKeyError (OtherPartyIdNotFoundError)
        let foreign_key_query = r#"mutation InsertOutboundShipment($input: InsertOutboundShipmentInput!) {
          insertOutboundShipment(input: $input) {
              ... on InsertOutboundShipmentError {
                error {
                  ... on ForeignKeyError {
                    __typename
                    key
                  }
                }
              }
          }
        }"#;
        let variables = Some(json!({
          "input": {
            "id": "ci_insert_1",
            "otherPartyId": "not existing",
          }
        }));
        let expected = json!({
            "insertOutboundShipment": {
              "error": {
                "__typename": "ForeignKeyError",
                "key": "OTHER_PARTY_ID"
              }
            }
          }
        );
        assert_gql_query(&settings, foreign_key_query, &variables, &expected).await;

        // Test succeeding insert
        let variables = Some(json!({
          "input": {
            "id": "ci_insert_1",
            "otherPartyId": other_party_customer.id,
            "comment": "ci comment"
          }
        }));
        let expected = json!({
            "insertOutboundShipment": {
              "id": "ci_insert_1",
              "otherPartyId": other_party_customer.id,
              "type": "OUTBOUND_SHIPMENT",
              "comment": "ci comment",
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;
        // make sure item has been inserted
        invoice_repository.find_one_by_id("ci_insert_1").unwrap();

        // RecordAlreadyExist,
        let variables = Some(json!({
          "input": {
            "id": "ci_insert_1",
            "otherPartyId": other_party_customer.id,
          }
        }));
        let expected = json!({
            "insertOutboundShipment": {
              "error": {
                "__typename": "RecordAlreadyExist"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;
    }
}