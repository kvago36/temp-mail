use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub type MailId = Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSession {
    pub mail_id: Uuid,
    pub email: String,
    pub created_ad: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mail {
    pub id: String,
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
