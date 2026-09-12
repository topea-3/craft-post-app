ALTER TABLE postcard_sends ADD COLUMN source TEXT NOT NULL DEFAULT 'print'
  CHECK (source IN ('print', 'manual'));

ALTER TABLE postcard_sends ADD COLUMN memo TEXT;
