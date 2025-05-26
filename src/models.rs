use serde::Serialize;
use std::fmt;

#[derive(Serialize)]
pub struct Mail {
    sender: String,
    subject: String,
    message: String,
    timestamp: String,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "mailbox_status")]
#[sqlx(rename_all = "lowercase")]
pub enum MailboxStatus {
    New,
    Permanent,
    Expired,
}

impl fmt::Display for MailboxStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            MailboxStatus::New => "new",
            MailboxStatus::Expired => "expired",
            MailboxStatus::Permanent => "permanent",
        };
        write!(f, "{}", s)
    }
}

impl Mail {
    pub fn new(sender: String, subject: String, message: String, timestamp: String) -> Self {
        Mail {
            sender,
            subject,
            message,
            timestamp,
        }
    }
}
