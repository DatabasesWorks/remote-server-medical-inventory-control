use crate::database::schema::ItemPropertyRow;

pub fn mock_item_property_a() -> ItemPropertyRow {
    ItemPropertyRow {
        id: String::from("item_property_a"),
        key: String::from("item_property_a"),
        name: String::from("Item property A"),
    }
}

pub fn mock_item_property_b() -> ItemPropertyRow {
    ItemPropertyRow {
        id: String::from("item_property_b"),
        key: String::from("item_property_b"),
        name: String::from("Item property B"),
    }
}

pub fn mock_item_property_c() -> ItemPropertyRow {
    ItemPropertyRow {
        id: String::from("item_property_c"),
        key: String::from("item_property_c"),
        name: String::from("Item property C"),
    }
}

pub fn mock_item_properties() -> Vec<ItemPropertyRow> {
    vec![
        mock_item_property_a(),
        mock_item_property_b(),
        mock_item_property_c(),
    ]
}
