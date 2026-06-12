ALTER TABLE transactions ADD COLUMN is_investment BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX idx_txn_investment ON transactions(user_id, is_investment);
