mod graphql {
    use crate::graphql::assert_graphql_query;
    use domain::{
        location::{Location, LocationFilter, LocationSort},
        PaginationOption,
    };
    use repository::{mock::MockDataInserts, StorageConnectionManager};
    use serde_json::json;
    use server::test_utils::setup_all;
    use service::{
        location::LocationServiceTrait,
        service_provider::{ServiceContext, ServiceProvider},
        ListError, ListResult,
    };

    type GetLocations = dyn Fn(
            Option<PaginationOption>,
            Option<LocationFilter>,
            Option<LocationSort>,
        ) -> Result<ListResult<Location>, ListError>
        + Sync
        + Send;

    pub struct TestService(pub Box<GetLocations>);

    impl LocationServiceTrait for TestService {
        fn get_locations(
            &self,
            _: &ServiceContext,
            pagination: Option<PaginationOption>,
            filter: Option<LocationFilter>,
            sort: Option<LocationSort>,
        ) -> Result<ListResult<Location>, ListError> {
            (self.0)(pagination, filter, sort)
        }
    }

    pub fn service_provider(
        location_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.location_service = Box::new(location_service);
        service_provider
    }

    #[actix_rt::test]
    async fn test_graphql_locations_pagination() {
        let (_, _, connection_manager, settings) =
            setup_all("test_graphql_locations_pagination", MockDataInserts::all()).await;

        // Test errors
        let query = r#"
      query {
          locations {
            ... on ConnectorError {
              error {
                  ...on PaginationError {
                     rangeError {
                        description
                        field
                        max
                        min
                     }
                  }
              }
            }
          }
      }
      "#;

        // Test pagination, first over limit
        let test_service = TestService(Box::new(|_, _, _| Err(ListError::LimitAboveMax(1000))));

        let expected = json!({
              "locations": {
                "error": {
                  "rangeError": {
                    "description": "Value is above maximum",
                    "field": "first",
                    "max": 1000,
                    "min": null
                  }
                }
              }
          }
        );

        assert_graphql_query!(
            &settings,
            query,
            &None,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // Test pagination, first too small
        let test_service = TestService(Box::new(|_, _, _| Err(ListError::LimitBelowMin(1))));

        let expected = json!({
              "locations": {
                "error": {
                  "rangeError": {
                    "description": "Value is below minimum",
                    "field": "first",
                    "max": null,
                    "min": 1
                  }
                }
              }
          }
        );

        assert_graphql_query!(
            &settings,
            query,
            &None,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // Test success
        let query = r#"
      query(
          $page: PaginationInput
        ) {
          locations(page: $page) {
            __typename
          }
        }

      "#;

        let expected = json!({
              "locations": {
                  "__typename": "LocationConnector"
              }
          }
        );

        // Test pagination
        let test_service = TestService(Box::new(|page, _, _| {
            assert_eq!(
                page,
                Some(PaginationOption {
                    limit: Some(2),
                    offset: Some(1)
                })
            );
            Ok(ListResult::empty())
        }));

        let variables = json!({
          "page": {
            "first": 2,
            "offset": 1
          }
        });

        assert_graphql_query!(
            &settings,
            query,
            &Some(variables),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );
    }
}
