-- Add migration script here
CREATE TYPE mailbox_status AS ENUM ('new', 'permanent', 'expired');
