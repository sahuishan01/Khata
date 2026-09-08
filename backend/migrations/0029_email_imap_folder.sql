-- Bank statement mails are almost always auto-archived by Gmail filters, so a
-- search scoped to INBOX misses them. Default the scan to "All Mail"; the sync
-- worker falls back to INBOX when the folder does not exist (non-Gmail IMAP).

ALTER TABLE user_email_configs
    ADD COLUMN imap_folder text NOT NULL DEFAULT '[Gmail]/All Mail';
