use std::fmt;
use serde::Serialize;

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
}

impl fmt::Display for MailboxStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            MailboxStatus::New => "new",
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