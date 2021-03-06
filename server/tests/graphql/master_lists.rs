mod graphql {
    use crate::graphql::assert_graphql_query;
    use domain::{
        master_list::{MasterListFilter, MasterListSort},
        EqualFilter, PaginationOption, SimpleStringFilter,
    };
    use repository::{
        mock::{mock_master_list_master_list_line_filter_test, MockDataInserts},
        MasterList, StorageConnectionManager,
    };
    use serde_json::{json, Value};
    use server::test_utils::setup_all;
    use service::{
        master_list::MasterListServiceTrait,
        service_provider::{ServiceContext, ServiceProvider},
        ListError, ListResult,
    };

    type GetMasterLists = dyn Fn(
            Option<PaginationOption>,
            Option<MasterListFilter>,
            Option<MasterListSort>,
        ) -> Result<ListResult<MasterList>, ListError>
        + Sync
        + Send;

    pub struct TestService(pub Box<GetMasterLists>);

    impl MasterListServiceTrait for TestService {
        fn get_master_lists(
            &self,
            _: &ServiceContext,
            pagination: Option<PaginationOption>,
            filter: Option<MasterListFilter>,
            sort: Option<MasterListSort>,
        ) -> Result<ListResult<MasterList>, ListError> {
            (self.0)(pagination, filter, sort)
        }
    }

    pub fn service_provider(
        masterlist_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.master_list_service = Box::new(masterlist_service);
        service_provider
    }

    #[actix_rt::test]
    async fn test_graphql_masterlists_success() {
        let (_, _, connection_manager, settings) =
            setup_all("test_graphql_masterlists_success", MockDataInserts::all()).await;

        let query = r#"
        query {
            masterLists {
              ... on MasterListConnector {
                nodes {
                  id
                  name
                  code
                  description
                  lines {
                      nodes {
                          id
                          item {
                              id
                          }
                      }
                      totalCount
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
                rows: vec![MasterList {
                    id: "master_list_master_list_line_filter_test".to_owned(),
                    name: "test_name".to_owned(),
                    code: "test_code".to_owned(),
                    description: "test_description".to_owned(),
                }],
                count: 1,
            })
        }));

        // TODO would prefer for loaders to be using service provider
        // in which case we would override both item and master list line service
        // and test it's mapping here, rather then from mock data
        let mock_data_lines = &mock_master_list_master_list_line_filter_test().lines;

        let lines: Vec<Value> = mock_data_lines
            .iter()
            .map(|line| {
                json!({
                    "id": line.id,
                    "item": {
                        "id": line.item_id
                    }
                })
            })
            .collect();

        let expected = json!({
              "masterLists": {
                  "nodes": [
                      {
                          "id": "master_list_master_list_line_filter_test",
                          "name": "test_name",
                          "code": "test_code",
                          "description": "test_description",
                          "lines": {
                              "nodes": lines,
                              "totalCount": lines.len()
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
              "masterLists": {
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
    async fn test_graphql_masterlists_filters() {
        let (_, _, connection_manager, settings) =
            setup_all("test_graphql_masterlist_filters", MockDataInserts::all()).await;

        let query = r#"
        query(
            $filter: MasterListFilterInput
          ) {
            masterLists(filter: $filter) {
              __typename
            }
          }

        "#;

        let expected = json!({
              "masterLists": {
                  "__typename": "MasterListConnector"
              }
          }
        );

        // Test filter
        let test_service = TestService(Box::new(|_, filter, _| {
            assert_eq!(
                filter,
                Some(MasterListFilter {
                    id: Some(EqualFilter {
                        equal_to: Some("test_id_filter".to_owned()),
                        not_equal_to: None,
                        equal_any: None
                    }),
                    name: Some(SimpleStringFilter {
                        equal_to: Some("name_filter".to_owned()),
                        like: None
                    }),
                    code: Some(SimpleStringFilter {
                        equal_to: Some("code_filter".to_owned()),
                        like: None
                    }),
                    description: Some(SimpleStringFilter {
                        equal_to: Some("description_filter_1".to_owned()),
                        like: Some("description_filter_2".to_owned()),
                    }),
                    exists_for_name: Some(SimpleStringFilter {
                        equal_to: None,
                        like: Some("exists_for_name_filter".to_owned()),
                    }),
                    exists_for_name_id: Some(EqualFilter {
                        equal_to: None,
                        not_equal_to: Some("test_name_id_filter".to_owned()),
                        equal_any: None
                    })
                })
            );
            Ok(ListResult::empty())
        }));

        let variables = json!({
          "filter": {
            "id": { "equalTo": "test_id_filter"},
            "name": {"equalTo": "name_filter" },
            "code": {"equalTo": "code_filter" },
            "description": {"equalTo": "description_filter_1", "like": "description_filter_2" },
            "existsForName": {"like": "exists_for_name_filter" },
            "existsForNameId": {"notEqualTo": "test_name_id_filter"}
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
