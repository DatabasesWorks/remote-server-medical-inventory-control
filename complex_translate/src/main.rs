use serde::{Deserialize, Serialize};

#[derive(Debug)]
enum SyncType {
    Delete,
    Update,
    Insert,
}
#[derive(Debug)]
struct SyncRecord {
    sync_type: SyncType,
    record_type: String,
    data: String,
}

// Serde already does the renaming for us
#[derive(Deserialize, Serialize, Debug)]
struct SimpleTranslatedRecord {
    id: String,
    color: Option<String>,
    shape: Option<String>,
    some_other_value: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
struct DBRecord {
    id: String,
}
#[derive(Deserialize, Serialize, Debug)]
struct DBPropertyJoin {
    record_id: String,
    property_id: String,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
enum DBPropertyType {
    Colors,
    Shapes,
}
#[derive(Deserialize, Serialize, Debug)]
struct DBProperty {
    id: String,
    r#type: DBPropertyType,
    value: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct DBHandler {
    records: Vec<DBRecord>,
    properties: Vec<DBProperty>,
    property_joins: Vec<DBPropertyJoin>,
}

impl<'a> SimpleSyncTranslation<'a> for SimpleTranslatedRecord {
    type RecordType = SimpleTranslatedRecord;

    fn get_original_type() -> &'static str {
        "original_record"
    }
}

impl<'a> ComplexTranslation for SimpleTranslatedRecord {
    fn complex_translation(&self, db_handler: &DBHandler) -> Option<Vec<Query>> {
        // Needs to be way better structure but as an example
        let color = match &self.color {
            Some(color) => color,
            None => panic!("no color in record"),
        };
        let color_property = db_handler
            .properties
            .iter()
            .find(|property| property.r#type == DBPropertyType::Colors && &property.value == color)
            .unwrap();

        let shape = match &self.shape {
            Some(shape) => shape,
            None => panic!("no shape in record"),
        };
        let shape_property = db_handler
            .properties
            .iter()
            .find(|property| property.r#type == DBPropertyType::Shapes && &property.value == shape)
            .unwrap();

        // This is crude, but ok for example
        let delete_properties: Vec<&DBPropertyJoin> = db_handler
            .property_joins
            .iter()
            .filter(|property_join| {
                property_join.record_id == self.id
                    && (property_join.property_id != color_property.id
                        && property_join.property_id != shape_property.id)
            })
            .collect();

        return Some(vec![
            Query {
                query: "delete property joins -> ",
                record: Box::new(format!("{:?}", delete_properties)),
            },
            Query {
                query: "update or insert these ones -> ",
                record: Box::new(format!("{:?}", vec![color_property, shape_property])),
            },
        ]);
    }
}

impl Mutatable for SimpleTranslatedRecord {
    fn make_sync_mutation(&self, sync_type: &SyncType) -> Query {
        match sync_type {
            SyncType::Delete => Query {
                query: "this will be a DELETE query, not a string",
                record: Box::new(format!("{}", self.id)),
            },
            _ => Query {
                query: "this will be a UPDATE OR INSERT query, not a string",
                record: Box::new(format!(
                    "id: {} some_other_value: {:?}",
                    self.id, self.some_other_value
                )),
            },
        }
    }
}

trait MutableAndComplexTranslation: Mutatable + ComplexTranslation {}

fn get_simple_translation(
    sync_record: &SyncRecord,
) -> Result<Box<dyn MutableAndComplexTranslation>, String> {
    if let Some(record) = SimpleTranslatedRecord::try_translate(&sync_record) {
        return Ok(Box::new(record));
    }

    Err("Cannot find matching translation".to_string())
}

#[derive(Debug)]
struct Query {
    query: &'static str,
    record: Box<dyn std::fmt::Debug>,
}

fn main() {
    // Insert or Update original_record -> SimpleTranslatedRecord
    let sync_record = SyncRecord {
        sync_type: SyncType::Insert,
        record_type: "original_record".to_owned(),
        data: r#"{
            "id": "ABC",
            "color": "green",
            "shape": "round",
            "some_other_value": "value"
        }"#
        .to_owned(),
    };

    let db_handler: DBHandler = serde_json::from_str(
        r#"
        {
            "records": [
                {
                    "id": "ABC"
                }
            ],
            "properties": [
                {
                    "id": "fp_o",
                    "type": "Colors",
                    "value": "orange"
                },
                {
                    "id": "fp_g",
                    "type": "Colors",
                    "value": "green"
                },
                {
                    "id": "sp_r",
                    "type": "Shapes",
                    "value": "round"
                },
                {
                    "id": "sp_s",
                    "type": "Shapes",
                    "value": "square"
                }
            ],
            "property_joins": [
                {
                    "record_id": "ABC",
                    "property_id": "fp_o"
                },
                {
                    "record_id": "ABC",
                    "property_id": "sp_s"
                }
            ]
        }
    "#,
    )
    .unwrap();

    let translated_record = get_simple_translation(&sync_record).unwrap();

    let mut queries = Vec::new();
    queries.push(translated_record.make_sync_mutation(&sync_record.sync_type));

    if let Some(mut complex_translations) = translated_record.complex_translation(&db_handler) {
        queries.append(&mut complex_translations);
    }
    println!("{:#?}", queries);

    // With no record
    let sync_record = SyncRecord {
        sync_type: SyncType::Insert,
        record_type: "original_record".to_owned(),
        data: r#"{
            "id": "ABC",
            "color": "green",
            "shape": "round",
            "some_other_value": "value"
        }"#
        .to_owned(),
    };

    let db_handler: DBHandler = serde_json::from_str(
        r#"
        {
            "records": [
         
            ],
            "properties": [
                {
                    "id": "fp_o",
                    "type": "Colors",
                    "value": "orange"
                },
                {
                    "id": "fp_g",
                    "type": "Colors",
                    "value": "green"
                },
                {
                    "id": "sp_r",
                    "type": "Shapes",
                    "value": "round"
                },
                {
                    "id": "sp_s",
                    "type": "Shapes",
                    "value": "square"
                }
            ],
            "property_joins": [

            ]
        }
    "#,
    )
    .unwrap();

    let translated_record = get_simple_translation(&sync_record).unwrap();

    let mut queries = Vec::new();
    queries.push(translated_record.make_sync_mutation(&sync_record.sync_type));

    if let Some(mut complex_translations) = translated_record.complex_translation(&db_handler) {
        queries.append(&mut complex_translations);
    }
    println!("{:#?}", queries);
}

trait Mutatable {
    fn make_sync_mutation(&self, sync_type: &SyncType) -> Query;
}
trait SimpleSyncTranslation<'a> {
    type RecordType: Deserialize<'a> + Mutatable + SimpleSyncTranslation<'a> + ComplexTranslation;
    fn get_original_type() -> &'static str;

    fn try_translate(sync_record: &'a SyncRecord) -> Option<Self::RecordType> {
        match sync_record.record_type.as_str() {
            record_type if record_type == Self::RecordType::get_original_type() => {
                Some(serde_json::from_str::<'a, Self::RecordType>(&sync_record.data).unwrap())
            }

            _ => None,
        }
    }
}

impl<T> MutableAndComplexTranslation for T where T: Mutatable + ComplexTranslation {}
trait ComplexTranslation {
    fn complex_translation(&self, db_handler: &DBHandler) -> Option<Vec<Query>> {
        None
    }
}
