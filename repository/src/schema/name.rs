use super::diesel_schema::name_table;

#[derive(Clone, Queryable, Insertable, Debug, PartialEq, Eq, AsChangeset)]
#[table_name = "name_table"]
pub struct NameRow {
    pub id: String,
    pub name: String,
    pub code: String,
    pub is_customer: bool,
    pub is_supplier: bool,
}