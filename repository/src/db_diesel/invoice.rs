use super::StorageConnection;

use crate::{
    repository_error::RepositoryError,
    schema::{InvoiceRow, InvoiceRowType},
};

use diesel::prelude::*;

pub struct InvoiceRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> InvoiceRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        InvoiceRepository { connection }
    }

    #[cfg(feature = "postgres")]
    pub fn upsert_one(&self, row: &InvoiceRow) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::invoice::dsl::*;
        diesel::insert_into(invoice)
            .values(row)
            .on_conflict(id)
            .do_update()
            .set(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(not(feature = "postgres"))]
    pub fn upsert_one(&self, row: &InvoiceRow) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::invoice::dsl::*;
        diesel::replace_into(invoice)
            .values(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub fn delete(&self, invoice_id: &str) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::invoice::dsl::*;
        diesel::delete(invoice.filter(id.eq(invoice_id))).execute(&self.connection.connection)?;
        Ok(())
    }

    pub fn find_one_by_id(&self, invoice_id: &str) -> Result<InvoiceRow, RepositoryError> {
        use crate::schema::diesel_schema::invoice::dsl::*;
        let result = invoice
            .filter(id.eq(invoice_id))
            .first(&self.connection.connection);
        result.map_err(|err| RepositoryError::from(err))
    }

    pub fn find_many_by_id(&self, ids: &[String]) -> Result<Vec<InvoiceRow>, RepositoryError> {
        use crate::schema::diesel_schema::invoice::dsl::*;
        let result = invoice
            .filter(id.eq_any(ids))
            .load(&self.connection.connection)?;
        Ok(result)
    }
}

pub struct OutboundShipmentRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> OutboundShipmentRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        OutboundShipmentRepository { connection }
    }

    pub async fn find_many_by_name_id(
        &self,
        name: &str,
    ) -> Result<Vec<InvoiceRow>, RepositoryError> {
        use crate::schema::diesel_schema::invoice::dsl::*;
        let result = invoice
            .filter(
                type_
                    .eq(InvoiceRowType::OutboundShipment)
                    .and(name_id.eq(name)),
            )
            .get_results(&self.connection.connection)?;
        Ok(result)
    }

    pub fn find_many_by_store_id(&self, store: &str) -> Result<Vec<InvoiceRow>, RepositoryError> {
        use crate::schema::diesel_schema::invoice::dsl::*;
        let result = invoice
            .filter(
                type_
                    .eq(InvoiceRowType::OutboundShipment)
                    .and(store_id.eq(store)),
            )
            .get_results(&self.connection.connection)?;
        Ok(result)
    }
}
