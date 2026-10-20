-- Required by sync_push's ON CONFLICT target. NULL ids from statement imports
-- remain allowed; a device transaction is unique within its owning user.
CREATE UNIQUE INDEX idx_transactions_client_user ON transactions (client_id, user_id);
