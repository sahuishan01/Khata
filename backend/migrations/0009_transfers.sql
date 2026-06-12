-- User's own accounts (UPI handles, account numbers, names)
CREATE TABLE user_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    identifier TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, identifier)
);

-- Mark transactions as transfers (excluded from spend/earn/savings)
ALTER TABLE transactions ADD COLUMN is_transfer BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX idx_txn_transfer ON transactions(user_id, is_transfer);
