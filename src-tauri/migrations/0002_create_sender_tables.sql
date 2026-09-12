-- Craft Post App - sender domain tables

CREATE TABLE IF NOT EXISTS sender_entries (
  id                TEXT PRIMARY KEY,
  label             TEXT    NOT NULL,
  primary_last      TEXT    NOT NULL,
  primary_first     TEXT    NOT NULL,
  primary_kana_last TEXT,
  primary_kana_first TEXT,
  postal_code       TEXT    NOT NULL,
  prefecture        TEXT    NOT NULL,
  city              TEXT    NOT NULL,
  street            TEXT    NOT NULL,
  building          TEXT,
  phone_number      TEXT,
  archived          INTEGER NOT NULL DEFAULT 0,
  created_at        TEXT    NOT NULL,
  updated_at        TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sender_entries_archived_updated_at
  ON sender_entries (archived, updated_at DESC);

CREATE TABLE IF NOT EXISTS sender_co_recipients (
  id              TEXT PRIMARY KEY,
  sender_entry_id TEXT    NOT NULL,
  order_index     INTEGER NOT NULL,
  last            TEXT    NOT NULL,
  first           TEXT    NOT NULL,
  kana_last       TEXT,
  kana_first      TEXT,
  archived        INTEGER NOT NULL DEFAULT 0,
  created_at      TEXT    NOT NULL,
  updated_at      TEXT    NOT NULL,
  FOREIGN KEY (sender_entry_id) REFERENCES sender_entries(id)
);

CREATE INDEX IF NOT EXISTS idx_sender_co_recipients_entry_order
  ON sender_co_recipients (sender_entry_id, order_index);

CREATE TABLE IF NOT EXISTS sender_address_links (
  id               TEXT PRIMARY KEY,
  sender_entry_id  TEXT    NOT NULL,
  address_entry_id TEXT    NOT NULL,
  created_at       TEXT    NOT NULL,
  updated_at       TEXT    NOT NULL,
  FOREIGN KEY (sender_entry_id) REFERENCES sender_entries(id),
  FOREIGN KEY (address_entry_id) REFERENCES address_entries(id)
);

CREATE INDEX IF NOT EXISTS idx_sender_address_links_address
  ON sender_address_links (address_entry_id);

CREATE INDEX IF NOT EXISTS idx_sender_address_links_sender
  ON sender_address_links (sender_entry_id);

