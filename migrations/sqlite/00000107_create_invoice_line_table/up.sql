CREATE TABLE invoice_line (
    id TEXT NOT NULL PRIMARY KEY,
    invoice_id TEXT NOT NULL REFERENCES invoice(id),
    item_id TEXT NOT NULL REFERENCES item(id),
    stock_line_id TEXT REFERENCES stock_line(id),
    batch TEXT,
    expiry_date TIMESTAMP,
    cost_price_per_pack REAL NOT NULL,
    sell_price_per_pack REAL NOT NULL,
    total_after_tax REAL NOT NULL,
    number_of_packs INTEGER NOT NULL,
    pack_size INTEGER NOT NULL
);
