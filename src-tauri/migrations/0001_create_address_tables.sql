-- Craft Post App - address domain tables

CREATE TABLE IF NOT EXISTS address_entries (
  id                TEXT PRIMARY KEY,
  primary_last      TEXT    NOT NULL,
  primary_first     TEXT    NOT NULL,
  primary_kana_last TEXT,
  primary_kana_first TEXT,

  honorific         TEXT    NOT NULL,
  postal_code       TEXT    NOT NULL,

  prefecture        TEXT    NOT NULL,
  city              TEXT    NOT NULL,
  street            TEXT    NOT NULL,
  building          TEXT,

  memo              TEXT,
  archived          INTEGER NOT NULL DEFAULT 0,
  created_at        TEXT    NOT NULL,
  updated_at        TEXT    NOT NULL,

  CHECK (honorific IN ('様', '御中', 'ご家族様', 'なし'))
);

CREATE INDEX IF NOT EXISTS idx_address_entries_archived_updated_at
  ON address_entries (archived, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_address_entries_name
  ON address_entries (primary_last, primary_first);

CREATE INDEX IF NOT EXISTS idx_address_entries_postal_code
  ON address_entries (postal_code);

CREATE TABLE IF NOT EXISTS address_co_recipients (
  id               TEXT PRIMARY KEY,
  address_entry_id TEXT    NOT NULL,
  order_index      INTEGER NOT NULL,

  last             TEXT    NOT NULL,
  first            TEXT    NOT NULL,
  kana_last        TEXT,
  kana_first       TEXT,

  archived         INTEGER NOT NULL DEFAULT 0,
  created_at       TEXT    NOT NULL,
  updated_at       TEXT    NOT NULL,

  FOREIGN KEY (address_entry_id) REFERENCES address_entries(id)
);

CREATE INDEX IF NOT EXISTS idx_address_co_recipients_entry_order
  ON address_co_recipients (address_entry_id, order_index);

