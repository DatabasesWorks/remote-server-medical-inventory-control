use serde_json::Value as JsonValue;

#[derive(Clone)]
pub struct ItemPropertyJoinRow {
    pub id: String,
    pub item_id: String,
    pub property_id: String,
    pub value: JsonValue,
}
