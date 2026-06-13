-- Allow 'investment' as a valid txn_type in categories
ALTER TABLE categories DROP CONSTRAINT IF EXISTS categories_txn_type_check;
ALTER TABLE categories ADD CONSTRAINT categories_txn_type_check CHECK (txn_type IN ('income', 'expense', 'investment'));

-- Remove is_investment from transactions (now derived from categories)
DROP INDEX IF EXISTS idx_txn_investment;
ALTER TABLE transactions DROP COLUMN is_investment;
