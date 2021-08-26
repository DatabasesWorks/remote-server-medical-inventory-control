use crate::database::repository::RepositoryError;
use crate::database::schema::{DatabaseRow, ItemPropertyRow};

use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ItemPropertyRepository {
    mock_data: Arc<Mutex<HashMap<String, DatabaseRow>>>,
}

impl ItemPropertyRepository {
    pub fn new(mock_data: Arc<Mutex<HashMap<String, DatabaseRow>>>) -> ItemPropertyRepository {
        ItemPropertyRepository { mock_data }
    }

    #[allow(dead_code)]
    pub async fn insert_one(&self, item_property: &ItemPropertyRow) -> Result<(), RepositoryError> {
        info!(
            "Inserting item_property record (item_property.id={})",
            item_property.id
        );
        self.mock_data.lock().unwrap().insert(
            item_property.id.to_string(),
            DatabaseRow::ItemProperty(item_property.clone()),
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn find_all(&self) -> Result<Vec<ItemPropertyRow>, RepositoryError> {
        let filter_item_property = |row: &DatabaseRow| -> Option<ItemPropertyRow> {
            if let DatabaseRow::ItemProperty(item_property) = row {
                Some(item_property.clone())
            } else {
                None
            }
        };

        Ok(self
            .mock_data
            .lock()
            .unwrap()
            .values()
            .filter_map(filter_item_property)
            .collect())
    }


    #[allow(dead_code)]
    pub async fn find_one_by_id(&self, id: &str) -> Result<ItemPropertyRow, RepositoryError> {
        info!("Querying item_property record (item_property.id={})", id);
        match self.mock_data.lock().unwrap().get(&id.to_string()) {
            Some(DatabaseRow::ItemProperty(item_property)) => Ok(item_property.clone()),
            _ => Err(RepositoryError {
                msg: String::from(format!(
                    "Failed to find item_propeerty record (item_property.id={})",
                    id
                )),
            }),
        }
    }

    #[allow(dead_code)]
    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<ItemPropertyRow>, RepositoryError> {
        info!(
            "Querying multiple item_property records (item_property.id={:?})",
            ids
        );
        let mut item_properties = vec![];
        ids.iter().for_each(|id| {
            if let Some(DatabaseRow::ItemProperty(item_property)) =
                self.mock_data.lock().unwrap().get(id)
            {
                item_properties.push(item_property.clone());
            }
        });
        Ok(item_properties)
    }
}
