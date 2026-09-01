-- Read receipts (open tracking) for sent emails.
--
-- A unique token is generated per outgoing message and embedded as a tracking
-- pixel URL in the message HTML. When the pixel is fetched, the open is
-- recorded by token on the sender's copy of the message.
ALTER TABLE email_messages
    ADD COLUMN IF NOT EXISTS open_tracking_token uuid,
    ADD COLUMN IF NOT EXISTS first_opened_at timestamptz,
    ADD COLUMN IF NOT EXISTS last_opened_at timestamptz,
    ADD COLUMN IF NOT EXISTS open_count integer NOT NULL DEFAULT 0;

CREATE UNIQUE INDEX IF NOT EXISTS email_messages_open_tracking_token_idx
    ON email_messages (open_tracking_token)
    WHERE open_tracking_token IS NOT NULL;

-- Per-inbox opt-out. Read receipts are enabled by default, matching #3943.
ALTER TABLE email_settings
    ADD COLUMN IF NOT EXISTS read_receipts_enabled boolean NOT NULL DEFAULT true;
