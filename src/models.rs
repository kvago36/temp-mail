use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Mail {
    pub subject: String,
    pub receivers: Vec<String>,
    pub sender: String,
    pub message: String,
    pub timestamp: i64,
    pub body: String,
    pub attachments: Vec<String>,
    pub domain: String,
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
