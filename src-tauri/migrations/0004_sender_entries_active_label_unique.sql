-- active（archived_at IS NULL）な差出人の label を DB レベルで一意に保つ
CREATE UNIQUE INDEX IF NOT EXISTS idx_sender_entries_active_label_unique
  ON sender_entries (label) WHERE archived_at IS NULL;
