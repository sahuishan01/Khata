CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    statement_id UUID NOT NULL REFERENCES statements(id) ON DELETE CASCADE,
    bank TEXT NOT NULL,
    account_label TEXT NOT NULL DEFAULT '',
    txn_date DATE NOT NULL,
    value_date DATE NOT NULL,
    description TEXT NOT NULL,
    raw_description TEXT NOT NULL,
    amount NUMERIC(14,2) NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('debit','credit')),
    balance NUMERIC(14,2),
    bank_ref TEXT,
    fingerprint TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, fingerprint)
);
