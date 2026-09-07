-- Per-run history for the email ingestion worker + IMAP watermark / filter
-- columns on the per-user config.

CREATE TABLE email_sync_runs (
    id                 uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id            uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at         timestamptz NOT NULL DEFAULT now(),
    finished_at        timestamptz,
    status             text NOT NULL DEFAULT 'running',   -- running | ok | error
    trigger            text NOT NULL,                     -- poll | manual
    full_scan          boolean NOT NULL DEFAULT false,
    messages_scanned   integer NOT NULL DEFAULT 0,
    attachments_seen   integer NOT NULL DEFAULT 0,
    attachments_parsed integer NOT NULL DEFAULT 0,
    txns_imported      integer NOT NULL DEFAULT 0,
    txns_skipped       integer NOT NULL DEFAULT 0,
    errors             jsonb NOT NULL DEFAULT '[]'::jsonb, -- [{stage, item, detail}]
    error              text                                -- fatal summary when status = 'error'
);

CREATE INDEX idx_email_runs_user ON email_sync_runs (user_id, started_at DESC);

-- RLS enabled but not FORCEd, matching user_email_configs: the table-owning
-- application role bypasses the policy (needed by the background worker, which
-- has no per-request user context), while any non-owner path is still isolated.
ALTER TABLE email_sync_runs ENABLE ROW LEVEL SECURITY;
CREATE POLICY email_runs_user_iso ON email_sync_runs
    USING (user_id = (current_setting('app.current_user_id', true))::uuid);

ALTER TABLE user_email_configs
    ADD COLUMN sender_allowlist  text[],
    ADD COLUMN subject_patterns  text[],
    ADD COLUMN imap_uid_validity bigint,
    ADD COLUMN last_uid          bigint;
