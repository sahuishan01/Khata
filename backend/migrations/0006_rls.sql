-- Enable RLS; FORCE makes even the table owner subject to the policy
ALTER TABLE transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
ALTER TABLE statements ENABLE ROW LEVEL SECURITY;
ALTER TABLE statements FORCE ROW LEVEL SECURITY;
ALTER TABLE chat_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE chat_messages FORCE ROW LEVEL SECURITY;

-- Policy: row is visible only when app.current_user_id matches
CREATE POLICY txn_user_iso ON transactions
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
CREATE POLICY stmt_user_iso ON statements
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
CREATE POLICY chat_user_iso ON chat_messages
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);

-- Read-only role gets SELECT only
GRANT SELECT ON transactions  TO khata_ro;
GRANT SELECT ON statements    TO khata_ro;
GRANT SELECT ON chat_messages TO khata_ro;
