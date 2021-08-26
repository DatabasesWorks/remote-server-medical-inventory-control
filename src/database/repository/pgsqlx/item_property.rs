use crate::database::repository::RepositoryError;
use crate::database::schema::ItemPropertyRow;

#[derive(Clone)]
pub struct ItemPropertyRepository {
    pool: sqlx::PgPool,
}

impl ItemPropertyRepository {
    #[allow(dead_code)]
    pub fn new(pool: sqlx::PgPool) -> ItemPropertyRepository {
        ItemPropertyRepository { pool }
    }

    #[allow(dead_code)]
    pub async fn insert_one(&self, item_property: &ItemPropertyRow) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO item_property (id, key, name)
            VALUES ($1, $2, $3)
            "#,
            item_property.id,
            item_property.key,
            item_property.name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn find_all(&self) -> Result<Vec<ItemPropertyRow>, RepositoryError> {
        let item_properties = sqlx::query_as!(
            ItemPropertyRow,
            r#"
            SELECT id, key, name
            FROM item_property
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(item_properties)
    }

    #[allow(dead_code)]
    pub async fn find_one_by_id(&self, id: &str) -> Result<ItemPropertyRow, RepositoryError> {
        let item_property = sqlx::query_as!(
            ItemPropertyRow,
            r#"
            SELECT id, key, name
            FROM item_property
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(item_property)
    }

    #[allow(dead_code)]
    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<ItemPropertyRow>, RepositoryError> {
        let item_properties = sqlx::query_as!(
            ItemPropertyRow,
            r#"
            SELECT id, key, name
            FROM item_property
            WHERE id = ANY($1)
            "#,
            ids
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(item_properties)
    }
}
