-- TOP-35: 年賀状送付対象フラグ。1 = 対象、0 = 対象外。既存行は対象。

ALTER TABLE address_entries
  ADD COLUMN nenga_send_target INTEGER NOT NULL DEFAULT 1
  CHECK (nenga_send_target IN (0, 1));

CREATE INDEX IF NOT EXISTS idx_address_entries_active_nenga_send_target
  ON address_entries (nenga_send_target)
  WHERE archived_at IS NULL;
