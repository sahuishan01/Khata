-- Migration 0027: Per-User Encrypted Email Ingestion (Gmail/IMAP)

CREATE TABLE IF NOT EXISTS user_email_configs (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    email_address TEXT NOT NULL,
    encrypted_app_password TEXT NOT NULL, -- AES-256-GCM encrypted base64 string
    encrypted_pdf_password TEXT,         -- Optional encrypted PDF statement password
    imap_server TEXT NOT NULL DEFAULT 'imap.gmail.com:993',
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    last_synced_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enable Row Level Security (RLS) for multi-tenant isolation
ALTER TABLE user_email_configs ENABLE ROW LEVEL SECURITY;

CREATE POLICY email_config_user_iso ON user_email_configs
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);
