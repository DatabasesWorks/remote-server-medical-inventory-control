-- Create item_property_join table

CREATE TABLE item_property_join (
    id varchar(255) NOT NULL PRIMARY KEY,
    item_id varchar(255) NOT NULL,
    property_id varchar(255) NOT NULL,
    value jsonb NOT NULL
)
