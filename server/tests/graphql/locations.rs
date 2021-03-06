mod graphql {
    use crate::graphql::assert_graphql_query;
    use domain::{
        location::{Location, LocationFilter, LocationSort, LocationSortField},
        EqualFilter, PaginationOption, Sort,
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
    async fn test_graphql_locations_success() {
        let (_, _, connection_manager, settings) =
            setup_all("test_graphql_locations_success", MockDataInserts::all()).await;

        let query = r#"
        query {
            locations {
              ... on LocationConnector {
                nodes {
                  id
                  name
                  code
                  onHold
                  stock {
                      ... on StockLineConnector {
                          nodes {
                              id
                          }
                      }
                  }
                }
                totalCount
              }
            }
        }
        "#;

        // Test single record
        let test_service = TestService(Box::new(|_, _, _| {
            Ok(ListResult {
                rows: vec![Location {
                    id: "location_on_hold".to_owned(),
                    name: "test_name".to_owned(),
                    code: "test_code".to_owned(),
                    on_hold: true,
                }],
                count: 1,
            })
        }));

        let expected = json!({
              "locations": {
                  "nodes": [
                      {
                          "id": "location_on_hold",
                          "name": "test_name",
                          "code": "test_code",
                          "onHold": true,
                          "stock": {
                              "nodes": [
                                  {
                                      "id": "stock_line_location_is_on_hold"
                                  }
                              ]
                          }
                      },
                  ],
                  "totalCount": 1
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

        // Test no records

        let test_service = TestService(Box::new(|_, _, _| {
            Ok(ListResult {
                rows: Vec::new(),
                count: 0,
            })
        }));

        let expected = json!({
              "locations": {
                  "nodes": [

                  ],
                  "totalCount": 0
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
    }

    #[actix_rt::test]
    async fn test_graphql_locations_inputs() {
        let (_, _, connection_manager, settings) =
            setup_all("test_graphql_location_inputs", MockDataInserts::all()).await;

        let query = r#"
        query(
            $sort: [LocationSortInput]
            $filter: LocationFilterInput
          ) {
            locations(sort: $sort, filter: $filter) {
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

        // Test sort by name no desc
        let test_service = TestService(Box::new(|_, _, sort| {
            assert_eq!(
                sort,
                Some(Sort {
                    key: LocationSortField::Name,
                    desc: None
                })
            );
            Ok(ListResult::empty())
        }));

        let variables = json!({
          "sort": [{
            "key": "name",
          }]
        });

        assert_graphql_query!(
            &settings,
            query,
            &Some(variables),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // Test sort by code with desc
        let test_service = TestService(Box::new(|_, _, sort| {
            assert_eq!(
                sort,
                Some(Sort {
                    key: LocationSortField::Code,
                    desc: Some(true)
                })
            );
            Ok(ListResult::empty())
        }));

        let variables = json!({
          "sort": [{
            "key": "code",
            "desc": true
          }]
        });

        assert_graphql_query!(
            &settings,
            query,
            &Some(variables),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // Test filter
        let test_service = TestService(Box::new(|_, filter, _| {
            assert_eq!(
                filter,
                Some(LocationFilter::new().name(EqualFilter::equal_to("match_name")))
            );
            Ok(ListResult::empty())
        }));

        let variables = json!({
          "filter": {
            "name": { "equalTo": "match_name"},
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
