ALTER TABLE transactions ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
CREATE INDEX idx_txn_version ON transactions(user_id, version);
