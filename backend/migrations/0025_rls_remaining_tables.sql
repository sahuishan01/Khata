-- Enable RLS on remaining user-scoped tables
-- Using ENABLE (not FORCE) so the table owner can still insert during tests/migrations.
-- The application always sets app.current_user_id in transactions.
ALTER TABLE user_accounts        ENABLE ROW LEVEL SECURITY;
ALTER TABLE budgets              ENABLE ROW LEVEL SECURITY;
ALTER TABLE category_rules       ENABLE ROW LEVEL SECURITY;
ALTER TABLE portfolio_assets     ENABLE ROW LEVEL SECURITY;
ALTER TABLE portfolio_liabilities ENABLE ROW LEVEL SECURITY;
ALTER TABLE categories           ENABLE ROW LEVEL SECURITY;

-- Policy: row visible only when app.current_user_id matches
CREATE POLICY acct_user_iso ON user_accounts
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
CREATE POLICY budget_user_iso ON budgets
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
CREATE POLICY rule_user_iso ON category_rules
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
CREATE POLICY asset_user_iso ON portfolio_assets
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
CREATE POLICY liab_user_iso ON portfolio_liabilities
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
CREATE POLICY cat_user_iso ON categories
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
