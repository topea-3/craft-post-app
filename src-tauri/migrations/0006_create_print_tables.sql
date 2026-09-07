CREATE TABLE IF NOT EXISTS print_layout_preferences (
  id                  TEXT PRIMARY KEY,
  postcard_type       TEXT NOT NULL CHECK (postcard_type IN ('nenga', 'mochu')),
  layer_id            TEXT NOT NULL,
  offset_x_pt         REAL NOT NULL DEFAULT 0,
  offset_y_pt         REAL NOT NULL DEFAULT 0,
  updated_at          TEXT NOT NULL,
  UNIQUE (postcard_type, layer_id)
);

CREATE TABLE IF NOT EXISTS postcard_sends (
  id                  TEXT PRIMARY KEY,
  print_job_id        TEXT NOT NULL,           -- 同一 PDF ジョブの idempotency キー
  address_entry_id    TEXT NOT NULL,
  sender_entry_id     TEXT NOT NULL,
  sender_snapshot     TEXT NOT NULL,           -- JSON
  address_snapshot    TEXT NOT NULL,           -- JSON
  postcard_type       TEXT NOT NULL CHECK (postcard_type IN ('nenga', 'mochu')),
  sent_on             TEXT NOT NULL,           -- chrono::Local::now().date_naive() 'YYYY-MM-DD'
  created_at          TEXT NOT NULL,           -- ISO 8601 datetime
  updated_at          TEXT NOT NULL,
  deleted_at          TEXT,
  FOREIGN KEY (address_entry_id) REFERENCES address_entries(id),
  FOREIGN KEY (sender_entry_id) REFERENCES sender_entries(id)
);

CREATE INDEX IF NOT EXISTS idx_postcard_sends_active_sent_on
  ON postcard_sends (sent_on DESC) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_postcard_sends_active_address
  ON postcard_sends (address_entry_id, sent_on DESC) WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_postcard_sends_job_address
  ON postcard_sends (print_job_id, address_entry_id) WHERE deleted_at IS NULL;
