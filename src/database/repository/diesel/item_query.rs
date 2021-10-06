use super::{get_connection, DBBackendConnection, LeftJoinBoxedStatementFourWay};
use crate::{
    database::{
        repository::RepositoryError,
        schema::{
            diesel_schema::{
                item, item::dsl as item_dsl, master_list, master_list::dsl as master_list_dsl,
                master_list_line, master_list_line::dsl as master_list_line_dsl,
                master_list_name_join, master_list_name_join::dsl as master_list_name_join_dsl,
            },
            ItemRow, MasterListLineRow, MasterListNameJoinRow, MasterListRow,
        },
    },
    domain::{
        item::{Item, ItemFilter, ItemSort, ItemSortField},
        Pagination,
    },
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

pub type ItemQuery = (
    ItemRow,
    Option<(
        MasterListLineRow,
        Option<(MasterListRow, Option<MasterListNameJoinRow>)>,
    )>,
);

impl From<ItemQuery> for Item {
    fn from((item_row, master_list_line_option): ItemQuery) -> Self {
        let mut is_visible = false;
        if let Some((_, master_list_option)) = master_list_line_option {
            if let Some((_, master_list_name_join_option)) = master_list_line_option {
                if let Some(_) = master_list_name_join_option {
                    is_visible = true;
                }
            }
        }
        Item {
            id: item_row.id,
            name: item_row.name,
            code: item_row.code,
            is_visible,
        }
    }
}

pub struct ItemQueryRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl ItemQueryRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> ItemQueryRepository {
        ItemQueryRepository { pool }
    }

    pub fn count(&self, filter: &Option<ItemFilter>) -> Result<i64, RepositoryError> {
        // TODO (beyond M1), check that store_id matches current store
        let connection = get_connection(&self.pool)?;

        let query = create_filtered_query(filter);

        Ok(query.count().get_result(&*connection)?)
    }
    pub fn query(
        &self,
        pagination: Pagination,
        filter: &Option<ItemFilter>,
        sort: Option<ItemSort>,
    ) -> Result<Vec<Item>, RepositoryError> {
        let connection = get_connection(&self.pool)?;

        let mut query = create_filtered_query(filter);

        if let Some(sort) = sort {
            match sort.key {
                ItemSortField::Name => {
                    if sort.desc.unwrap_or(false) {
                        query = query.order(item_dsl::name.desc());
                    } else {
                        query = query.order(item_dsl::name.asc());
                    }
                }
                ItemSortField::Code => {
                    if sort.desc.unwrap_or(false) {
                        query = query.order(item_dsl::code.desc());
                    } else {
                        query = query.order(item_dsl::code.asc());
                    }
                }
            }
        } else {
            query = query.order(item_dsl::id.asc())
        }

        let result = query
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .load::<ItemQuery>(&*connection)?;

        Ok(result.into_iter().map(Item::from).collect())
    }
}

pub fn create_filtered_query<'a, 'b: 'a>(
    filter: &'b Option<ItemFilter>,
) -> LeftJoinBoxedStatementFourWay<
    'a,
    item::table,
    master_list_line::table,
    master_list::table,
    master_list_name_join::table,
> {
    let mut query = item_dsl::item
        .left_join(
            master_list_line_dsl::master_list_line.left_join(
                master_list_dsl::master_list
                    .left_join(master_list_name_join_dsl::master_list_name_join),
            ),
        )
        .into_boxed();

    if let Some(f) = filter {
        if let Some(code) = &f.code {
            if let Some(eq) = &code.equal_to {
                query = query.filter(item_dsl::code.eq(eq));
            } else if let Some(like) = &code.like {
                query = query.filter(item_dsl::code.like(format!("%{}%", like)));
            }
        }
        if let Some(name) = &f.name {
            if let Some(eq) = &name.equal_to {
                query = query.filter(item_dsl::name.eq(eq));
            } else if let Some(like) = &name.like {
                query = query.filter(item_dsl::name.like(format!("%{}%", like)));
            }
        }
        if let Some(is_visible) = f.is_visible.as_ref().map(|v| v.equal_to).flatten() {
            if is_visible {
                query = query.filter(master_list_name_join_dsl::id.is_not_null());
            } else {
                query = query.filter(master_list_name_join_dsl::id.is_null());
            }
        }
    }
    query
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::{
        database::{
            repository::{
                repository::{
                    MasterListLineRepository, MasterListNameJoinRepository, MasterListRepository,
                },
                ItemQueryRepository, ItemRepository, NameRepository,
            },
            schema::{ItemRow, MasterListLineRow, MasterListNameJoinRow, MasterListRow, NameRow},
        },
        util::test_db,
    };
    // TODO this is very repetitive, although it's ok for tests to be 'wet' I think we can do better (and still have readable tests)
    fn data() -> Vec<ItemRow> {
        let mut rows = Vec::new();
        for index in 0..200 {
            rows.push(ItemRow {
                id: format!("id{:05}", index),
                name: format!("name{}", index),
                code: format!("code{}", index),
            });
        }
        rows
    }

    #[actix_rt::test]
    async fn test_item_query_repository() {
        // Prepare
        let (pool, _, connection) = test_db::setup_all("test_item_query_repository", false).await;
        let item_query_repository = ItemQueryRepository::new(pool.clone());

        let rows = data();
        for row in rows.iter() {
            ItemRepository::upsert_one_tx(&connection, &row).unwrap();
        }

        let default_page_size = usize::try_from(DEFAULT_PAGE_SIZE).unwrap();

        // Test
        // .count()
        assert_eq!(
            usize::try_from(item_query_repository.count(None).unwrap()).unwrap(),
            rows.len()
        );

        // .all, no pagenation (default)
        assert_eq!(
            item_query_repository
                .all(&None, &None, &None)
                .unwrap()
                .len(),
            default_page_size
        );

        // .all, pagenation (offset 10)
        let result = item_query_repository
            .all(
                &Some(Pagination {
                    offset: Some(10),
                    first: None,
                }),
                &None,
                &None,
            )
            .unwrap();
        assert_eq!(result.len(), default_page_size);
        assert_eq!(result[0].0, rows[10]);
        assert_eq!(
            result[default_page_size - 1].0,
            rows[10 + default_page_size - 1]
        );

        // .all, pagenation (first 10)
        let result = item_query_repository
            .all(
                &Some(Pagination {
                    offset: None,
                    first: Some(10),
                }),
                &None,
                &None,
            )
            .unwrap();
        assert_eq!(result.len(), 10);
        assert_eq!((*result.last().unwrap()).0, rows[9]);

        // .all, pagenation (offset 150, first 90) <- more then records in table
        let result = item_query_repository
            .all(
                &Some(Pagination {
                    offset: Some(150),
                    first: Some(90),
                }),
                &None,
                &None,
            )
            .unwrap();
        assert_eq!(result.len(), rows.len() - 150);
        assert_eq!((*result.last().unwrap()).0, (*rows.last().unwrap()));
    }

    // TODO not sure where this fits, seems like this unit test has a lot of dependencies
    // I think test snapshot-like functionality is need ?

    // Really wanted to test visibility join, so added here for now

    #[actix_rt::test]
    async fn test_item_query_repository_visibility() {
        // Prepare
        let (pool, _, connection) =
            test_db::setup_all("test_item_query_repository_visibility", false).await;
        let item_query_repository = ItemQueryRepository::new(pool.clone());

        let item_rows = vec![
            ItemRow {
                id: "item1".to_owned(),
                name: "name1".to_owned(),
                code: "name1".to_owned(),
            },
            ItemRow {
                id: "item2".to_owned(),
                name: "name2".to_owned(),
                code: "name2".to_owned(),
            },
            ItemRow {
                id: "item3".to_owned(),
                name: "name3".to_owned(),
                code: "name3".to_owned(),
            },
            ItemRow {
                id: "item4".to_owned(),
                name: "name4".to_owned(),
                code: "name4".to_owned(),
            },
            ItemRow {
                id: "item5".to_owned(),
                name: "name5".to_owned(),
                code: "name5".to_owned(),
            },
        ];

        let master_list_rows = vec![
            MasterListRow {
                id: "master_list1".to_owned(),
                name: "".to_owned(),
                code: "".to_owned(),
                description: "".to_owned(),
            },
            MasterListRow {
                id: "master_list2".to_owned(),
                name: "".to_owned(),
                code: "".to_owned(),
                description: "".to_owned(),
            },
        ];

        let master_list_line_rows = vec![
            MasterListLineRow {
                id: "id1".to_owned(),
                item_id: "item1".to_owned(),
                master_list_id: "master_list1".to_owned(),
            },
            MasterListLineRow {
                id: "id2".to_owned(),
                item_id: "item2".to_owned(),
                master_list_id: "master_list1".to_owned(),
            },
            MasterListLineRow {
                id: "id3".to_owned(),
                item_id: "item3".to_owned(),
                master_list_id: "master_list2".to_owned(),
            },
            MasterListLineRow {
                id: "id4".to_owned(),
                item_id: "item4".to_owned(),
                master_list_id: "master_list2".to_owned(),
            },
        ];

        let name_row = NameRow {
            id: "name1".to_owned(),
            name: "".to_owned(),
            code: "".to_owned(),
            is_supplier: true,
            is_customer: true,
        };

        let master_list_name_join_1 = MasterListNameJoinRow {
            id: "id1".to_owned(),
            name_id: "name1".to_owned(),
            master_list_id: "master_list1".to_owned(),
        };

        let master_list_name_join_2 = MasterListNameJoinRow {
            id: "id2".to_owned(),
            name_id: "name1".to_owned(),
            master_list_id: "master_list2".to_owned(),
        };

        for row in item_rows.iter() {
            ItemRepository::upsert_one_tx(&connection, row).unwrap();
        }

        for row in master_list_rows {
            MasterListRepository::upsert_one_tx(&connection, &row).unwrap();
        }

        for row in master_list_line_rows {
            MasterListLineRepository::upsert_one_tx(&connection, &row).unwrap();
        }

        NameRepository::upsert_one_tx(&connection, &name_row).unwrap();
        // Test

        // Before adding any joins
        let results0: Vec<ItemRow> = item_query_repository
            .all(&None, &None, &None)
            .unwrap()
            .into_iter()
            .map(|v| v.0)
            .collect();
        assert_eq!(results0, item_rows);

        // After adding first join (item1 and item2 visible)
        MasterListNameJoinRepository::upsert_one_tx(&connection, &master_list_name_join_1).unwrap();
        let results = item_query_repository.all(&None, &None, &None).unwrap();
        assert!(results[0].2.is_some());
        assert!(results[1].2.is_some());

        // After adding second join (item3 and item4 visible)
        MasterListNameJoinRepository::upsert_one_tx(&connection, &master_list_name_join_2).unwrap();
        let results = item_query_repository.all(&None, &None, &None).unwrap();
        assert!(results[2].2.is_some());
        assert!(results[3].2.is_some());

        // test is_visible filter:
        let results = item_query_repository
            .all(
                &None,
                &Some(ItemFilter {
                    name: None,
                    code: None,
                    // query invisible rows
                    is_visible: Some(EqualFilter {
                        equal_to: Some(false),
                    }),
                }),
                &None,
            )
            .unwrap();
        assert_eq!(results[0].0, item_rows[4]);
        // get visible rows
        let results = item_query_repository
            .all(
                &None,
                &Some(ItemFilter {
                    name: None,
                    code: None,
                    // query invisible rows
                    is_visible: Some(EqualFilter {
                        equal_to: Some(true),
                    }),
                }),
                &None,
            )
            .unwrap();
        assert_eq!(results.len(), 4);
    }
}
