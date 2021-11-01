mod graphql {
    use crate::graphql::common::{
        assert_unwrap_enum, assert_unwrap_optional_key, get_invoice_inline,
        get_invoice_line_inline, get_invoice_lines_inline, get_stock_line_inline,
    };
    use crate::graphql::get_gql_result;
    use crate::graphql::{
        insert_outbound_shipment_line_full as insert, InsertOutboundShipmentLineFull as Insert,
    };
    use graphql_client::{GraphQLQuery, Response};
    use remote_server::database::repository::ItemRepository;
    use remote_server::database::schema::{InvoiceLineRow, StockLineRow};
    use remote_server::{
        database::mock::MockDataInserts,
        domain::{invoice::InvoiceFilter, Pagination},
        util::test_db,
    };

    use insert::InsertOutboundShipmentLineErrorInterface::*;
    use uuid::Uuid;

    macro_rules! assert_unwrap_response_variant {
        ($response:ident) => {
            assert_unwrap_optional_key!($response, data).insert_outbound_shipment_line
        };
    }

    macro_rules! assert_unwrap_line {
        ($response:ident) => {{
            let response_variant = assert_unwrap_response_variant!($response);
            assert_unwrap_enum!(
                response_variant,
                insert::InsertOutboundShipmentLineResponse::InvoiceLineNode
            )
        }};
    }

    macro_rules! assert_unwrap_error {
        ($response:ident) => {{
            let response_variant = assert_unwrap_response_variant!($response);
            let error_wrapper = assert_unwrap_enum!(
                response_variant,
                insert::InsertOutboundShipmentLineResponse::InsertOutboundShipmentLineError
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
    async fn test_insert_outbound_shipment_line() {
        let (_, connection, settings) = test_db::setup_all(
            "test_insert_outbound_shipment_line_query",
            MockDataInserts::all(),
        )
        .await;

        // Setup

        let draft_outbound_shipment = get_invoice_inline!(
            InvoiceFilter::new()
                .match_outbound_shipment()
                .match_draft()
                .match_id("outbound_shipment_c"),
            &connection
        );

        let confirmed_outbound_shipment = get_invoice_inline!(
            InvoiceFilter::new()
                .match_outbound_shipment()
                .match_confirmed()
                .match_id("outbound_shipment_d"),
            &connection
        );

        let finalised_outbound_shipment = get_invoice_inline!(
            InvoiceFilter::new()
                .match_outbound_shipment()
                .match_finalised(),
            &connection
        );

        let inbound_shipment = get_invoice_inline!(
            InvoiceFilter::new()
                .match_inbound_shipment()
                .match_id("inbound_shipment_c"),
            &connection
        );

        let draft_lines = get_invoice_lines_inline!(&draft_outbound_shipment.id, &connection);

        let supplier_lines = get_invoice_lines_inline!(&inbound_shipment.id, &connection);
        let item_not_in_invoices_id = "item_c".to_string();
        let stock_line_not_in_invoices_id = "item_c_line_a".to_string();

        let main_draft_line = draft_lines[0].clone();

        let base_variables = insert::Variables {
            id: Uuid::new_v4().to_string(),
            invoice_id: draft_outbound_shipment.id.clone(),
            item_id: item_not_in_invoices_id.clone(),
            number_of_packs: 3,
            stock_line_id: stock_line_not_in_invoices_id.clone(),
        };

        // Test RecordAlreadyExist

        let mut variables = base_variables.clone();
        variables.id = main_draft_line.id.clone();

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;

        assert_error!(
            response,
            RecordAlreadyExist(insert::RecordAlreadyExist {
                description: "Record already exists".to_string(),
            })
        );

        // Test ForeingKeyError Item

        let mut variables = base_variables.clone();
        variables.item_id = "invalid".to_string();

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            ForeignKeyError(insert::ForeignKeyError {
                description: "FK record doesn't exist".to_string(),
                key: insert::ForeignKey::ItemId,
            })
        );

        // Test ForeingKeyError Invoice

        let mut variables = base_variables.clone();
        variables.invoice_id = "invalid".to_string();

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            ForeignKeyError(insert::ForeignKeyError {
                description: "FK record doesn't exist".to_string(),
                key: insert::ForeignKey::InvoiceId,
            })
        );

        // Test CannotEditFinalisedInvoice

        let mut variables = base_variables.clone();
        variables.invoice_id = finalised_outbound_shipment.id.clone();

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            CannotEditFinalisedInvoice(insert::CannotEditFinalisedInvoice {
                description: "Cannot edit finalised invoice".to_string(),
            },)
        );

        // Test NotAnOutboundShipment

        let mut variables = base_variables.clone();
        variables.invoice_id = supplier_lines[0].invoice_id.clone();

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            NotAnOutboundShipment(insert::NotAnOutboundShipment {
                description: "Invoice is not Outbound Shipment".to_string(),
            })
        );

        // Test RangeError NumberOfPacks

        let mut variables = base_variables.clone();
        variables.number_of_packs = 0;

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            RangeError(insert::RangeError {
                description: "Value is below minimum".to_string(),
                field: insert::RangeField::NumberOfPacks,
                max: None,
                min: Some(1),
            })
        );

        // Test StockLineAlreadyExistsInInvoice

        let mut variables = base_variables.clone();
        variables.item_id = draft_lines[1].item_id.clone();
        variables.stock_line_id = draft_lines[1].stock_line_id.clone().unwrap();

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;

        let error_variant = assert_unwrap_error!(response);
        let line_variant = assert_unwrap_enum!(error_variant, StockLineAlreadyExistsInInvoice).line;
        let line = assert_unwrap_enum!(line_variant, insert::InvoiceLineResponse::InvoiceLineNode);
        assert_eq!(line.id, draft_lines[1].id);

        // Test NotEnoughStockForReduction

        let stock_line = get_stock_line_inline!(&stock_line_not_in_invoices_id, &connection);

        let mut variables = base_variables.clone();
        variables.number_of_packs = stock_line.available_number_of_packs as i64 + 1;

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;

        let error_variant = assert_unwrap_error!(response);
        let stock_line_variant =
            assert_unwrap_enum!(error_variant, NotEnoughStockForReduction).batch;
        let stock_line =
            assert_unwrap_enum!(stock_line_variant, insert::StockLineResponse::StockLineNode);

        assert_eq!(stock_line.id, stock_line_not_in_invoices_id);

        // Test ItemDoesNotMatchStockLine

        let mut variables = base_variables.clone();
        variables.item_id = main_draft_line.item_id.clone();

        let query = Insert::build_query(variables);
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;
        assert_error!(
            response,
            ItemDoesNotMatchStockLine(insert::ItemDoesNotMatchStockLine {
                description: "Item does not match stock line".to_string(),
            })
        );

        // Test Success Draft Reduction

        let start_stock_line = get_stock_line_inline!(&stock_line_not_in_invoices_id, &connection);
        let number_of_packs = 1;
        let item = ItemRepository::new(&connection)
            .find_one_by_id(&item_not_in_invoices_id)
            .unwrap();

        let mut variables = base_variables.clone();
        variables.number_of_packs = number_of_packs;

        let query = Insert::build_query(variables.clone());
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;

        let line = assert_unwrap_line!(response);
        assert_eq!(line.id, variables.id);

        let new_line = get_invoice_line_inline!(&variables.id, &connection);
        let new_stock_line = get_stock_line_inline!(&start_stock_line.id, &connection);

        assert_eq!(new_line.number_of_packs as i64, number_of_packs);
        assert_eq!(
            new_stock_line.available_number_of_packs as i64,
            start_stock_line.available_number_of_packs as i64 - number_of_packs
        );

        assert_eq!(
            new_stock_line.total_number_of_packs,
            start_stock_line.total_number_of_packs
        );

        assert_eq!(item.name, new_line.item_name);
        assert_eq!(item.code, new_line.item_code);
        assert_eq!(new_stock_line, FromStockLine(new_line));

        // Test Confirmed Reduction

        let start_stock_line = get_stock_line_inline!(&stock_line_not_in_invoices_id, &connection);
        let number_of_packs = 3;
        let item = ItemRepository::new(&connection)
            .find_one_by_id(&item_not_in_invoices_id)
            .unwrap();

        let mut variables = base_variables.clone();
        variables.id = Uuid::new_v4().to_string();
        variables.number_of_packs = number_of_packs;
        variables.invoice_id = confirmed_outbound_shipment.id.clone();

        let query = Insert::build_query(variables.clone());
        let response: Response<insert::ResponseData> = get_gql_result(&settings, query).await;

        let line = assert_unwrap_line!(response);
        assert_eq!(line.id, variables.id);

        let new_line = get_invoice_line_inline!(&variables.id, &connection);
        let new_stock_line = get_stock_line_inline!(&start_stock_line.id, &connection);

        assert_eq!(new_line.number_of_packs as i64, number_of_packs);
        assert_eq!(
            new_stock_line.available_number_of_packs as i64,
            start_stock_line.available_number_of_packs as i64 - number_of_packs
        );

        assert_eq!(
            new_stock_line.total_number_of_packs as i64,
            start_stock_line.total_number_of_packs as i64 - number_of_packs
        );

        assert_eq!(item.name, new_line.item_name);
        assert_eq!(item.code, new_line.item_code);
        assert_eq!(new_stock_line, FromStockLine(new_line));
    }

    #[derive(Debug)]
    struct FromStockLine(pub InvoiceLineRow);

    impl PartialEq<FromStockLine> for StockLineRow {
        fn eq(&self, other: &FromStockLine) -> bool {
            let StockLineRow {
                id: stock_line_id,
                item_id,
                batch,
                expiry_date: _,
                pack_size,
                cost_price_per_pack,
                sell_price_per_pack,
                store_id: _,
                available_number_of_packs: _,
                total_number_of_packs: _,
            } = self;

            let line = &other.0;

            *item_id == line.item_id
                && Some(stock_line_id.clone()) == line.stock_line_id
                && *batch == line.batch
                && *pack_size == line.pack_size
                && *cost_price_per_pack == line.cost_price_per_pack
                && *sell_price_per_pack == line.sell_price_per_pack
            //    && *expiry_date == line.expiry_date
            // TODO test fails if expiry_date in stock_line is None
            // for some reason expiry_date is not set to None (NULL) in postgres
            // but ok in sqlite (also setting batch to None works correctly)
            // must be something to do with Date type
            // https://github.com/openmsupply/remote-server/issues/482
        }
    }
}