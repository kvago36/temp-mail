use chrono::{DateTime, Utc};
use std::hash::Hash;

use mail::email::Email;

#[derive(Debug)]
pub enum ContentTransferEncoding {
    Base64,
}

#[derive(Debug)]
pub enum ContentDisposition {
    Inline,
    Attachment(String),
}

#[derive(Debug)]
pub enum ContentType {
    TextPlain,
    TextHtml,
    MultipartAlternative(String),
    MultipartMixed(String),
    MultipartRelated(String),
    ApplicationPdf,
    ApplicationImage,
}

#[derive(Debug)]
struct Attachments {
    content_type: ContentType,
    content_disposition: ContentDisposition,
    content_transfer_encoding: ContentTransferEncoding,
    content: String,
}

#[derive(Debug)]
pub(crate) struct Message {
    date: DateTime<Utc>,
    from: Option<Email>,
    subject: String,
    to: Option<Email>,
    content_type: Option<ContentType>,
    mime_version: Option<String>,
    attachments: Vec<Attachments>,
}

impl Message {
    pub fn new() -> Self {
        let datetime_str = "1983 Apr 13 12:09:14.274 +0000";
        let datetime = DateTime::parse_from_str(datetime_str, "%Y %b %d %H:%M:%S%.3f %z").unwrap();

        Message {
            from: None,
            subject: "".to_owned(),
            to: None,
            attachments: Vec::new(),
            date: datetime.to_utc(),
            content_type: None,
            mime_version: None,
        }
    }

    pub fn add_attachment(&mut self, attachment: Attachments) {
        self.attachments.push(attachment);
    }

    pub fn set_mime_version(&mut self, version: String) {
        self.mime_version = Some(version)
    }

    pub fn get_content_type(&self) -> &Option<ContentType> {
        &self.content_type
    }

    pub fn set_content_type(&mut self, content_type: ContentType) {
        self.content_type = Some(content_type)
    }

    pub fn set_from(&mut self, from: &str) {
        if let Ok(email) = Email::new(from) {
            self.from = Some(email);
        }
    }
    pub fn set_to(&mut self, to: &str) {
        if let Ok(email) = Email::new(to) {
            self.to = Some(email);
        }
    }
    pub fn set_subject(&mut self, subject: String) {
        self.subject = subject;
    }
}
