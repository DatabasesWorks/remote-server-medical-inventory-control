-- Create item_property_join foreign key constraints

BEGIN;
ALTER TABLE item_property_join ADD CONSTRAINT fk_item FOREIGN KEY(item_id) REFERENCES item(id);
ALTER TABLE item_property_join ADD CONSTRAINT fk_property FOREIGN KEY(property_id) REFERENCES item_property(id);
COMMIT;
