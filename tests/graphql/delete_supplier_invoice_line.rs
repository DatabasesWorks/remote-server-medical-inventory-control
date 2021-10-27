mod graphql {
    use crate::graphql::common::{
        assert_matches, assert_unwrap_enum, assert_unwrap_optional_key, get_invoice_inline,
        get_invoice_lines_inline,
    };
    use crate::graphql::get_gql_result;
    use crate::graphql::{
        delete_supplier_invoice_line_full as delete, DeleteSupplierInvoiceLineFull as Delete,
    };

    use graphql_client::{GraphQLQuery, Response};
    use remote_server::database::repository::RepositoryError;
    use remote_server::{
        database::{
            mock::MockDataInserts,
            repository::{InvoiceLineRepository, StockLineRepository},
        },
        domain::{invoice::InvoiceFilter, Pagination},
        util::test_db,
    };

    use delete::DeleteSupplierInvoiceLineErrorInterface::*;

    macro_rules! assert_unwrap_response_variant {
        ($response:ident) => {
            assert_unwrap_optional_key!($response, data).delete_supplier_invoice_line
        };
    }

    macro_rules! assert_unwrap_delete {
        ($response:ident) => {{
            let response_variant = assert_unwrap_response_variant!($response);
            assert_unwrap_enum!(
                response_variant,
                delete::DeleteSupplierInvoiceLineResponse::DeleteResponse
            )
        }};
    }

    macro_rules! assert_unwrap_error {
        ($response:ident) => {{
            let response_variant = assert_unwrap_response_variant!($response);
            let error_wrapper = assert_unwrap_enum!(
                response_variant,
                delete::DeleteSupplierInvoiceLineResponse::DeleteSupplierInvoiceLineError
            );
            error_wrapper.error
        }};
    }

    macro_rules! assert_error {
        ($response:ident, $error:expr) => {{
            let lhs = assert_unwrap_error!($response);
            let rhs = $error;
            assert_eq!(lhs, rhs);
        }};
    }

    #[actix_rt::test]
    async fn test_delete_supplier_invoice_line() {
        let (_, connection, settings) = test_db::setup_all(
            "test_delete_supplier_invoice_line_query",
            MockDataInserts::all(),
        )
        .await;

        // Setup

        let draft_supplier_invoice = get_invoice_inline!(
            InvoiceFilter::new()
                .match_supplier_invoice()
                .match_draft()
                .match_id("supplier_invoice_c"),
            &connection
        );
        let confirmed_supplier_invoice = get_invoice_inline!(
            InvoiceFilter::new()
                .match_supplier_invoice()
                .match_confirmed()
                .match_id("supplier_invoice_d"),
            &connection
        );
        let finalised_supplier_invoice = get_invoice_inline!(
            InvoiceFilter::new()
                .match_supplier_invoice()
                .match_finalised(),
            &connection
        );
        let customer_invoice =
            get_invoice_inline!(InvoiceFilter::new().match_customer_invoice(), &connection);
        let confirmed_invoice_lines =
            get_invoice_lines_inline!(&confirmed_supplier_invoice.id.clone(), &connection);
        let customer_invoice_lines =
            get_invoice_lines_inline!(&customer_invoice.id.clone(), &connection);
        let finalised_invoice_lines =
            get_invoice_lines_inline!(&finalised_supplier_invoice.id.clone(), &connection);
        let draft_invoice_lines =
            get_invoice_lines_inline!(&draft_supplier_invoice.id.clone(), &connection);

        let base_variables = delete::Variables {
            id: draft_invoice_lines[0].id.clone(),
            invoice_id: draft_supplier_invoice.id.clone(),
        };

        // Test RecordNotFound Item

        let mut variables = base_variables.clone();
        variables.id = "invalid".to_string();

        let query = Delete::build_query(variables);
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;

        assert_error!(
            response,
            RecordNotFound(delete::RecordNotFound {
                description: "Record not found".to_string(),
            })
        );

        // Test ForeingKeyError Invoice

        let mut variables = base_variables.clone();
        variables.invoice_id = "invalid".to_string();

        let query = Delete::build_query(variables);
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            ForeignKeyError(delete::ForeignKeyError {
                description: "FK record doesn't exist".to_string(),
                key: delete::ForeignKey::InvoiceId,
            })
        );

        // Test CannotEditFinalisedInvoice

        let mut variables = base_variables.clone();
        variables.id = finalised_invoice_lines[0].id.clone();
        variables.invoice_id = finalised_supplier_invoice.id.clone();

        let query = Delete::build_query(variables);
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            CannotEditFinalisedInvoice(delete::CannotEditFinalisedInvoice {
                description: "Cannot edit finalised invoice".to_string(),
            },)
        );

        // Test NotASupplierInvoice

        let mut variables = base_variables.clone();
        variables.id = customer_invoice_lines[0].id.clone();
        variables.invoice_id = customer_invoice.id.clone();

        let query = Delete::build_query(variables);
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            NotASupplierInvoice(delete::NotASupplierInvoice {
                description: "Invoice is not Supplier Invoice".to_string(),
            })
        );

        // Test InvoiceLineBelongsToAnotherInvoice

        let mut variables = base_variables.clone();
        variables.invoice_id = confirmed_supplier_invoice.id.clone();

        let query = Delete::build_query(variables);
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;

        let error_variant = assert_unwrap_error!(response);
        let invoice_variant =
            assert_unwrap_enum!(error_variant, InvoiceLineBelongsToAnotherInvoice).invoice;
        let invoice = assert_unwrap_enum!(invoice_variant, delete::InvoiceResponse::InvoiceNode);
        assert_eq!(invoice.id, draft_supplier_invoice.id);

        // Test BatchIsReserved

        let mut variables = base_variables.clone();
        variables.id = confirmed_invoice_lines[1].id.clone();
        variables.invoice_id = confirmed_supplier_invoice.id.clone();
        let mut stock_line = StockLineRepository::new(&connection)
            .find_one_by_id(confirmed_invoice_lines[1].stock_line_id.as_ref().unwrap())
            .unwrap();
        stock_line.available_number_of_packs -= 1;
        StockLineRepository::new(&connection)
            .upsert_one(&stock_line)
            .unwrap();

        let query = Delete::build_query(variables);
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;

        assert_error!(
            response,
            BatchIsReserved(delete::BatchIsReserved {
                description: "Batch is already reserved/issued".to_string(),
            })
        );

        // Success Draft

        let variables = base_variables.clone();

        let query = Delete::build_query(variables.clone());
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;
        let delete_response = assert_unwrap_delete!(response);

        let deleted_line = InvoiceLineRepository::new(&connection).find_one_by_id(&variables.id);

        assert_eq!(
            delete_response,
            delete::DeleteResponse {
                id: variables.id.clone()
            }
        );

        assert!(matches!(deleted_line, Err(RepositoryError::NotFound)));

        // Success Confirmed

        let mut variables = base_variables.clone();
        variables.id = confirmed_invoice_lines[0].id.clone();
        variables.invoice_id = confirmed_supplier_invoice.id.clone();

        let query = Delete::build_query(variables.clone());
        let response: Response<delete::ResponseData> = get_gql_result(&settings, query).await;
        let delete_response = assert_unwrap_delete!(response);

        let deleted_line = InvoiceLineRepository::new(&connection).find_one_by_id(&variables.id);
        let deleted_stock_line = StockLineRepository::new(&connection)
            .find_one_by_id(&confirmed_invoice_lines[0].stock_line_id.clone().unwrap());

        assert_eq!(
            delete_response,
            delete::DeleteResponse {
                id: variables.id.clone()
            }
        );

        assert_matches!(deleted_line, Err(RepositoryError::NotFound));
        assert_matches!(deleted_stock_line, Err(RepositoryError::NotFound));
    }
}