#[cfg(test)]
mod query {
    use domain::{
        location::{DeleteLocation, LocationFilter},
        stock_line::StockLineFilter,
        EqualFilter,
    };
    use repository::{
        mock::MockDataInserts, test_db::setup_all, InvoiceLineRepository, LocationRepository,
        StockLineRepository, InvoiceLineFilter,
    };

    use crate::{
        current_store_id,
        location::delete::{DeleteLocationError, LocationInUse},
        service_provider::ServiceProvider,
    };

    #[actix_rt::test]
    async fn location_service_delete_errors() {
        let (_, _, connection_manager, _) =
            setup_all("location_service_delete_errors", MockDataInserts::all()).await;

        let connection = connection_manager.connection().unwrap();
        let location_repository = LocationRepository::new(&connection);
        let stock_line_repository = StockLineRepository::new(&connection);
        let invoice_line_repository = InvoiceLineRepository::new(&connection);

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.location_service;

        let locations_not_in_store = location_repository
            .query_by_filter(LocationFilter::new().store_id(EqualFilter::not_equal_to(
                &current_store_id(&connection).unwrap(),
            )))
            .unwrap();

        // Location does not exist
        assert_eq!(
            service.delete_location(
                &context,
                DeleteLocation {
                    id: "invalid".to_owned(),
                },
            ),
            Err(DeleteLocationError::LocationDoesNotExist)
        );

        // Location for another store
        assert_eq!(
            service.delete_location(
                &context,
                DeleteLocation {
                    id: locations_not_in_store[0].id.clone(),
                },
            ),
            Err(DeleteLocationError::LocationDoesNotBelongToCurrentStore)
        );

        // Location is not empty (invoice lines in use)
        let location_id = "location_1".to_owned();
        let stock_lines = stock_line_repository
            .query_by_filter(
                StockLineFilter::new().location_id(EqualFilter::equal_to(&location_id)),
            )
            .unwrap();
        let invoice_lines = invoice_line_repository
            .query_by_filter(
                InvoiceLineFilter::new().location_id(EqualFilter::equal_to(&location_id)),
            )
            .unwrap();

        assert_eq!(
            service.delete_location(&context, DeleteLocation { id: location_id }),
            Err(DeleteLocationError::LocationInUse(LocationInUse {
                stock_lines,
                invoice_lines
            }))
        );

        // Location is not empty (stock_lines in use)
        let location_id = "location_on_hold".to_owned();
        let stock_lines = stock_line_repository
            .query_by_filter(
                StockLineFilter::new().location_id(EqualFilter::equal_to(&location_id)),
            )
            .unwrap();
        let invoice_lines = invoice_line_repository
            .query_by_filter(
                InvoiceLineFilter::new().location_id(EqualFilter::equal_to(&location_id)),
            )
            .unwrap();

        assert_eq!(
            service.delete_location(&context, DeleteLocation { id: location_id }),
            Err(DeleteLocationError::LocationInUse(LocationInUse {
                stock_lines,
                invoice_lines
            }))
        );
    }
    #[actix_rt::test]
    async fn location_service_delete_success() {
        let (_, _, connection_manager, _) =
            setup_all("location_service_delete_success", MockDataInserts::all()).await;

        let connection = connection_manager.connection().unwrap();
        let location_repository = LocationRepository::new(&connection);
        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.location_service;

        assert_eq!(
            service.delete_location(
                &context,
                DeleteLocation {
                    id: "location_2".to_owned()
                },
            ),
            Ok("location_2".to_owned())
        );

        assert_eq!(
            location_repository
                .query_by_filter(LocationFilter::new().id(EqualFilter::equal_to("location_2")))
                .unwrap(),
            vec![]
        );
    }
}
