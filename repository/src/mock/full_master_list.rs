use std::collections::HashMap;

use crate::{
    db_diesel::{
        MasterListLineRowRepository, MasterListNameJoinRepository, MasterListRowRepository,
        StorageConnection,
    },
    schema::{MasterListLineRow, MasterListNameJoinRow, MasterListRow},
};

pub struct FullMockMasterList {
    pub master_list: MasterListRow,
    pub joins: Vec<MasterListNameJoinRow>,
    pub lines: Vec<MasterListLineRow>,
}

pub fn mock_master_list_item_query_test1() -> FullMockMasterList {
    FullMockMasterList {
        master_list: MasterListRow {
            id: "item_query_test1".to_owned(),
            name: "name_item_query_test1".to_owned(),
            code: "code_item_query_test1".to_owned(),
            description: "description_item_query_test1".to_owned(),
        },
        joins: vec![MasterListNameJoinRow {
            id: "item_query_test1".to_owned(),
            master_list_id: "item_query_test1".to_owned(),
            name_id: "name_store_a".to_owned(),
        }],
        lines: vec![MasterListLineRow {
            id: "item_query_test1".to_owned(),
            item_id: "item_query_test1".to_owned(),
            master_list_id: "item_query_test1".to_owned(),
        }],
    }
}

pub fn mock_master_list_master_list_filter_test() -> FullMockMasterList {
    FullMockMasterList {
        master_list: MasterListRow {
            id: "master_list_filter_test".to_owned(),
            name: "name_master_list_filter_test".to_owned(),
            code: "code_master_list_filter_test".to_owned(),
            description: "description_master_list_filter_test".to_owned(),
        },
        joins: vec![MasterListNameJoinRow {
            id: "master_list_filter_test".to_owned(),
            master_list_id: "master_list_filter_test".to_owned(),
            name_id: "id_master_list_filter_test".to_owned(),
        }],
        lines: Vec::new(),
    }
}

pub fn mock_master_list_master_list_line_filter_test() -> FullMockMasterList {
    FullMockMasterList {
        master_list: MasterListRow {
            id: "master_list_master_list_line_filter_test".to_owned(),
            name: "name_master_list_master_list_line_filter_test".to_owned(),
            code: "code_master_list_master_list_line_filter_test".to_owned(),
            description: "description_master_list_master_list_line_filter_test".to_owned(),
        },
        joins: Vec::new(),
        lines: vec![
            MasterListLineRow {
                id: "master_list_line_filter_test_1".to_owned(),
                item_id: "item_a".to_owned(),
                master_list_id: "master_list_master_list_line_filter_test".to_owned(),
            },
            MasterListLineRow {
                id: "master_list_line_filter_test_2".to_owned(),
                item_id: "item_b".to_owned(),
                master_list_id: "master_list_master_list_line_filter_test".to_owned(),
            },
        ],
    }
}

pub fn insert_full_mock_master_list(
    full_master_list: &FullMockMasterList,
    connection: &StorageConnection,
) {
    MasterListRowRepository::new(&connection)
        .upsert_one(&full_master_list.master_list)
        .unwrap();

    for line in full_master_list.lines.iter() {
        MasterListLineRowRepository::new(&connection)
            .upsert_one(line)
            .unwrap();
    }

    for join in full_master_list.joins.iter() {
        MasterListNameJoinRepository::new(&connection)
            .upsert_one(join)
            .unwrap();
    }
}

pub fn mock_full_master_list() -> HashMap<String, FullMockMasterList> {
    vec![
        (
            "item_query_test1".to_string(),
            mock_master_list_item_query_test1(),
        ),
        (
            "master_list_filter_test".to_string(),
            mock_master_list_master_list_filter_test(),
        ),
        (
            "master_list_master_list_line_filter_test".to_string(),
            mock_master_list_master_list_line_filter_test(),
        ),
    ]
    .into_iter()
    .collect()
}
