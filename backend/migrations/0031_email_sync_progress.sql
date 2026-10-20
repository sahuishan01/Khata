ALTER TABLE email_sync_runs
    ADD COLUMN phase text NOT NULL DEFAULT 'connecting',
    ADD COLUMN messages_total integer,
    ADD COLUMN has_more boolean NOT NULL DEFAULT false,
    ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();

UPDATE email_sync_runs SET phase = CASE status
    WHEN 'ok' THEN 'complete' WHEN 'error' THEN 'failed' ELSE 'connecting' END;
