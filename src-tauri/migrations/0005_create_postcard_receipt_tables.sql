CREATE TABLE IF NOT EXISTS postcard_receipts (
  id                  TEXT PRIMARY KEY,
  address_entry_id    TEXT,
  sender_display_name TEXT,
  received_at         TEXT NOT NULL,
  category            TEXT NOT NULL,
  memo                TEXT,
  deleted_at          TEXT,
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL,
  FOREIGN KEY (address_entry_id) REFERENCES address_entries(id)
);

CREATE INDEX IF NOT EXISTS idx_postcard_receipts_active_received_at
  ON postcard_receipts (received_at DESC) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_postcard_receipts_active_address
  ON postcard_receipts (address_entry_id, received_at DESC) WHERE deleted_at IS NULL;
