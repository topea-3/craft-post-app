-- TOP-34: 受取年・送付年の明示カラム

ALTER TABLE postcard_receipts ADD COLUMN receipt_year INTEGER NOT NULL DEFAULT 0;

UPDATE postcard_receipts
SET receipt_year = CAST(substr(received_at, 1, 4) AS INTEGER);

CREATE INDEX IF NOT EXISTS idx_postcard_receipts_active_receipt_year
  ON postcard_receipts (receipt_year DESC) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_postcard_receipts_active_mochu_year_address
  ON postcard_receipts (receipt_year, address_entry_id)
  WHERE deleted_at IS NULL AND category = 'mochu' AND address_entry_id IS NOT NULL;

ALTER TABLE postcard_sends ADD COLUMN send_year INTEGER NOT NULL DEFAULT 0;

UPDATE postcard_sends
SET send_year = CAST(substr(sent_on, 1, 4) AS INTEGER);

CREATE INDEX IF NOT EXISTS idx_postcard_sends_active_send_year
  ON postcard_sends (send_year DESC) WHERE deleted_at IS NULL;
