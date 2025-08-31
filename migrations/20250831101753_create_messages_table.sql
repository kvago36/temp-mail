-- Add migration script here
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mailbox_id UUID REFERENCES mailboxes(id) ON DELETE CASCADE,
    domain TEXT,
    client_ip TEXT,
    sender TEXT NOT NULL,
    subject TEXT,
    body TEXT,
    message TEXT,
    attachments TEXT[],
    received_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);