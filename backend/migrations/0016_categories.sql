CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    txn_type TEXT NOT NULL DEFAULT 'expense' CHECK (txn_type IN ('income', 'expense')),
    color TEXT NOT NULL DEFAULT '#6C5CE7',
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, name)
);
