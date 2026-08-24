-- Migration 0026: Subscriptions, Savings Goals, Tags, and Split Transactions

-- 1. Tags & Split Transactions on main transactions table
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}';
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS splits JSONB DEFAULT '[]';

-- 2. Subscriptions Table
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    amount NUMERIC(14,2) NOT NULL,
    billing_cycle TEXT NOT NULL CHECK (billing_cycle IN ('monthly', 'yearly', 'weekly')),
    next_due_date DATE NOT NULL,
    category TEXT NOT NULL DEFAULT 'Subscriptions',
    auto_detected BOOLEAN NOT NULL DEFAULT false,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_subscriptions_user ON subscriptions(user_id);

-- 3. Savings Goals Table
CREATE TABLE IF NOT EXISTS goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    target_amount NUMERIC(14,2) NOT NULL,
    current_amount NUMERIC(14,2) NOT NULL DEFAULT 0,
    target_date DATE,
    color_hex TEXT NOT NULL DEFAULT '#6366f1',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_goals_user ON goals(user_id);

-- Enable Row Level Security (RLS) for multi-tenant isolation
ALTER TABLE subscriptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE goals         ENABLE ROW LEVEL SECURITY;

CREATE POLICY sub_user_iso ON subscriptions
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);

CREATE POLICY goal_user_iso ON goals
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
