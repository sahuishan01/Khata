-- Opt-out flag for parsing bank *alert* emails (no attachment) into
-- transactions, alongside the existing statement-attachment ingestion.

ALTER TABLE user_email_configs
    ADD COLUMN parse_txn_emails boolean NOT NULL DEFAULT true;
