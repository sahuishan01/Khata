-- Migration 0026: Account Aggregator (Setu) Tables & RLS Policies

CREATE TABLE aa_sync_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    auto_fetch_enabled BOOLEAN NOT NULL DEFAULT true,
    fetch_interval_days INT NOT NULL DEFAULT 7 CHECK (fetch_interval_days IN (1, 3, 7, 14, 30)),
    last_fetched_at TIMESTAMPTZ,
    next_fetch_due_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE aa_consents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    consent_id TEXT NOT NULL UNIQUE,
    handle_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('PENDING', 'ACTIVE', 'REVOKED', 'EXPIRED', 'FAILED')),
    fi_types TEXT[] NOT NULL DEFAULT '{"DEPOSIT","MUTUAL_FUNDS","EQUITIES"}',
    valid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_aa_consents_user ON aa_consents(user_id);

-- Enable RLS for multi-tenant data isolation
ALTER TABLE aa_sync_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE aa_consents      ENABLE ROW LEVEL SECURITY;

CREATE POLICY aa_settings_user_iso ON aa_sync_settings
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);

CREATE POLICY aa_consents_user_iso ON aa_consents
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
