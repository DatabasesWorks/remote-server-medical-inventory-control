#![allow(where_clauses_object_safety)]

mod graphql {
    use crate::graphql::assert_gql_query;
    use remote_server::{
        database::{
            mock::{
                mock_invoice_lines, mock_invoices, mock_items, mock_names, mock_stock_lines,
                mock_stores,
            },
            repository::{
                get_repositories, InvoiceLineRepository, InvoiceRepository, ItemRepository,
                NameRepository, StockLineRepository, StorageConnectionManager, StoreRepository,
            },
            schema::InvoiceLineRow,
        },
        domain::stock_line::StockLine,
        util::test_db,
    };
    use serde_json::json;

    #[actix_rt::test]
    async fn test_graphql_customer_invoice_update() {
        let settings = test_db::get_test_settings("omsupply-database-gql-customer_invoice_update");
        test_db::setup(&settings.database).await;
        let repositories = get_repositories(&settings).await;
        let connection_manager = repositories.get::<StorageConnectionManager>().unwrap();
        let connection = connection_manager.connection().unwrap();

        // setup
        let name_repository = NameRepository::new(&connection);
        let store_repository = StoreRepository::new(&connection);
        let item_repository = ItemRepository::new(&connection);
        let stock_line_repository = StockLineRepository::new(&connection);
        let invoice_repository = InvoiceRepository::new(&connection);
        let invoice_line_repository = InvoiceLineRepository::new(&connection);
        let mock_names = mock_names();
        let mock_stores = mock_stores();
        let mock_items = mock_items();
        let mock_stock_lines = mock_stock_lines();
        let mock_invoices = mock_invoices();
        let mock_invoice_lines = mock_invoice_lines();
        for name in &mock_names {
            name_repository.insert_one(&name).await.unwrap();
        }
        for store in mock_stores {
            store_repository.insert_one(&store).await.unwrap();
        }
        for item in &mock_items {
            item_repository.upsert_one(item).unwrap();
        }
        for stock_line in &mock_stock_lines {
            stock_line_repository.upsert_one(stock_line).unwrap();
        }
        for invoice in &mock_invoices {
            invoice_repository.upsert_one(invoice).unwrap();
        }
        for invoice_line in &mock_invoice_lines {
            invoice_line_repository
                .insert_one(invoice_line)
                .await
                .unwrap();
        }

        let query = r#"mutation DeleteCustomerInvoice($input: UpdateCustomerInvoiceInput!) {
            updateCustomerInvoice(input: $input) {
                ... on UpdateCustomerInvoiceError {
                  error {
                    __typename
                  }
                }
                ... on InvoiceNode {
                  id
                  comment
                }
            }
        }"#;

        // CannotChangeStatusBackToDraftError
        let variables = Some(json!({
          "input": {
            "id": "customer_invoice_confirmed",
            "status": "DRAFT"
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "error": {
                "__typename": "CannotChangeStatusBackToDraftError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // FinalisedInvoiceIsNotEditableError
        let variables = Some(json!({
          "input": {
            "id": "customer_invoice_finalised",
            "status": "DRAFT"
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "error": {
                "__typename": "FinalisedInvoiceIsNotEditableError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // InvoiceNotFoundError
        let variables = Some(json!({
          "input": {
            "id": "does not exist",
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "error": {
                "__typename": "InvoiceNotFoundError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // ForeignKeyError (Other party does not exist)
        let variables = Some(json!({
          "input": {
            "id": "customer_invoice_a",
            "otherPartyId": "invalid_other_party"
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "error": {
                "__typename": "ForeignKeyError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // OtherPartyNotACustomerError
        let other_party_supplier = &mock_names[2];
        let variables = Some(json!({
          "input": {
            "id": "customer_invoice_a",
            "otherPartyId": other_party_supplier.id
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "error": {
                "__typename": "OtherPartyNotACustomerError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // NotACustomerInvoiceError
        let variables = Some(json!({
          "input": {
            "id": "supplier_invoice_a",
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "error": {
                "__typename": "NotACustomerInvoiceError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // InvoiceLineHasNoStockLineError
        let variables = Some(json!({
          "input": {
            "id": "customer_invoice_invalid_stock_line",
            "status": "FINALISED"
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "error": {
                "__typename": "InvoiceLineHasNoStockLineError"
              }
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;

        // helpers to compare totals
        let stock_lines_for_invoice_lines = |invoice_lines: &Vec<InvoiceLineRow>| {
            let stock_line_ids: Vec<String> = invoice_lines
                .iter()
                .filter_map(|invoice| invoice.stock_line_id.to_owned())
                .collect();
            stock_line_repository
                .find_many_by_ids(&stock_line_ids)
                .unwrap()
        };
        // calculates the expected stock line total for every invoice line row
        let expected_stock_line_totals = |invoice_lines: &Vec<InvoiceLineRow>| {
            let stock_lines = stock_lines_for_invoice_lines(invoice_lines);
            let expected_stock_line_totals: Vec<(StockLine, i32)> = stock_lines
                .into_iter()
                .map(|line| {
                    let invoice_line = invoice_lines
                        .iter()
                        .find(|il| il.stock_line_id.clone().unwrap() == line.id)
                        .unwrap();
                    let expected_total = line.total_number_of_packs - invoice_line.number_of_packs;
                    (line, expected_total)
                })
                .collect();
            expected_stock_line_totals
        };
        let assert_stock_line_totals =
            |invoice_lines: &Vec<InvoiceLineRow>, expected: &Vec<(StockLine, i32)>| {
                let stock_lines = stock_lines_for_invoice_lines(invoice_lines);
                for line in stock_lines {
                    let expected = expected.iter().find(|l| l.0.id == line.id).unwrap();
                    assert_eq!(line.total_number_of_packs, expected.1);
                }
            };

        // test DRAFT to CONFIRMED
        let invoice_lines = invoice_line_repository
            .find_many_by_invoice_id("customer_invoice_a")
            .unwrap();
        let expected_totals = expected_stock_line_totals(&invoice_lines);
        let variables = Some(json!({
          "input": {
            "id": "customer_invoice_a",
            "status": "CONFIRMED",
            "comment": "test_comment"
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "id": "customer_invoice_a",
              "comment": "test_comment"
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;
        assert_stock_line_totals(&invoice_lines, &expected_totals);

        // test DRAFT to FINALISED
        let invoice_lines = invoice_line_repository
            .find_many_by_invoice_id("customer_invoice_b")
            .unwrap();
        let expected_totals = expected_stock_line_totals(&invoice_lines);
        let variables = Some(json!({
          "input": {
            "id": "customer_invoice_b",
            "status": "FINALISED",
            "comment": "test_comment_b"
          }
        }));
        let expected = json!({
            "updateCustomerInvoice": {
              "id": "customer_invoice_b",
              "comment": "test_comment_b"
            }
          }
        );
        assert_gql_query(&settings, query, &variables, &expected).await;
        assert_stock_line_totals(&invoice_lines, &expected_totals);
    }
}
