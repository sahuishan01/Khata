CREATE INDEX idx_txn_user_date  ON transactions(user_id, value_date DESC);
CREATE INDEX idx_txn_user_dir   ON transactions(user_id, direction);
CREATE INDEX idx_stmt_user      ON statements(user_id);
CREATE INDEX idx_chat_user_date ON chat_messages(user_id, created_at DESC);
