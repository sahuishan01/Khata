ALTER TABLE transactions ADD COLUMN category TEXT NOT NULL DEFAULT 'Miscellaneous';
CREATE INDEX idx_txn_user_category ON transactions(user_id, category);
