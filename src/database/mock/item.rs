use crate::database::schema::ItemRow;

pub fn mock_item_a() -> ItemRow {
    ItemRow {
        id: String::from("item_a"),
        name: String::from("Item A"),
        code: String::from("code"),
    }
}

pub fn mock_item_b() -> ItemRow {
    ItemRow {
        id: String::from("item_b"),
        name: String::from("Item B"),
        code: String::from("code"),
    }
}

pub fn mock_item_c() -> ItemRow {
    ItemRow {
        id: String::from("item_c"),
        name: String::from("Item C"),
        code: String::from("code"),
    }
}

pub fn mock_items() -> Vec<ItemRow> {
    vec![mock_item_a(), mock_item_b(), mock_item_c()]
}
