-- A row's conflict revision is not a pagination cursor: many imported rows
-- share rev=1. Keep a separate monotonic change position for device downloads.
CREATE SEQUENCE transaction_sync_cursor;
ALTER TABLE transactions ADD COLUMN sync_rev bigint NOT NULL DEFAULT 0;
-- The table owner performs this backfill inside the migration transaction.
-- FORCE RLS otherwise hides existing users' rows from the startup connection.
ALTER TABLE transactions NO FORCE ROW LEVEL SECURITY;
UPDATE transactions SET sync_rev = nextval('transaction_sync_cursor');
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
CREATE INDEX idx_transactions_user_sync_cursor ON transactions (user_id, sync_rev);

CREATE FUNCTION assign_transaction_sync_cursor() RETURNS trigger AS $$
BEGIN
    -- Serialize writes within a user's transaction, so a later committed cursor
    -- cannot overtake an earlier uncommitted write for the same user.
    PERFORM id FROM users WHERE id = NEW.user_id FOR UPDATE;
    NEW.sync_rev := nextval('transaction_sync_cursor');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER transaction_sync_cursor_changed
BEFORE INSERT OR UPDATE ON transactions
FOR EACH ROW EXECUTE FUNCTION assign_transaction_sync_cursor();
