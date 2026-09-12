-- archived (INTEGER) -> archived_at (TEXT, RFC3339, NULL = 有効)

-- address_entries
ALTER TABLE address_entries ADD COLUMN archived_at TEXT;
UPDATE address_entries SET archived_at = updated_at WHERE archived = 1;
DROP INDEX IF EXISTS idx_address_entries_archived_updated_at;
ALTER TABLE address_entries DROP COLUMN archived;
CREATE INDEX IF NOT EXISTS idx_address_entries_active_updated_at
  ON address_entries (updated_at DESC) WHERE archived_at IS NULL;

-- address_co_recipients
ALTER TABLE address_co_recipients ADD COLUMN archived_at TEXT;
UPDATE address_co_recipients SET archived_at = updated_at WHERE archived = 1;
ALTER TABLE address_co_recipients DROP COLUMN archived;

-- sender_entries
ALTER TABLE sender_entries ADD COLUMN archived_at TEXT;
UPDATE sender_entries SET archived_at = updated_at WHERE archived = 1;
DROP INDEX IF EXISTS idx_sender_entries_archived_updated_at;
ALTER TABLE sender_entries DROP COLUMN archived;
CREATE INDEX IF NOT EXISTS idx_sender_entries_active_updated_at
  ON sender_entries (updated_at DESC) WHERE archived_at IS NULL;

-- sender_co_recipients
ALTER TABLE sender_co_recipients ADD COLUMN archived_at TEXT;
UPDATE sender_co_recipients SET archived_at = updated_at WHERE archived = 1;
ALTER TABLE sender_co_recipients DROP COLUMN archived;
