use chrono::{DateTime, Local};
use log::{debug, error, info, warn};
use mailparse::body::Body;
use mailparse::{MailHeader, ParsedMail};
use regex::Regex;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;

use crate::mail_proto::{Mail, MailRequest};

use crate::client_modules::request::Request;
use mail::email::Email;
use mail::error::MyError;

pub struct Payload {
    subject: String,
    from: Email,
    to: Vec<Email>,
    attachments: Vec<String>,
    body: Option<String>,
    message: Option<String>,
    timestamp: i64,
}

pub struct State {
    domain: String,
    sender: Option<Email>,
    recipients: Vec<Email>,
    channel: Sender<MailRequest>,
}

impl State {
    pub(crate) fn new(channel: Sender<MailRequest>) -> Self {
        State {
            channel,
            domain: "".to_owned(),
            sender: None,
            recipients: vec![Email::new("email16@test.com").unwrap()],
        }
    }

    pub(crate) async fn handle_request(&mut self, message: Request<'_>) -> Result<(), MyError> {
        match message {
            Request::Hello(domain) => {
                self.domain = domain;
            }
            Request::Mail(sender) => {
                self.sender = Some(sender);
            }
            Request::Recipient(recipient) => {
                self.recipients.push(recipient);
            }
            Request::Data(mail) => {
                let now: DateTime<Local> = Local::now();

                // DEBUG:
                self.sender = Some(Email::new("test@test.com")?);

                let mut p = Payload {
                    subject: "".to_string(),
                    from: self
                        .sender
                        .take()
                        .expect("Client should sent sender MAIL FROM"),
                    to: self.recipients.drain(..).collect(),
                    attachments: vec![],
                    body: None,
                    message: None,
                    timestamp: now.timestamp(),
                };

                for header in mail.headers {
                    match header.get_key().as_str() {
                        "From" => {
                            let value = header.get_value();
                            let from =
                                Email::new(&value).expect("Could not parse From email address");

                            p.from = from;
                        }
                        "To" => {
                            let value = header.get_value();

                            let emails = value
                                .split(',')
                                .filter_map(|s| Email::new(s).ok())
                                .collect::<Vec<_>>();

                            p.to = emails;
                        }
                        "Content-Type" => {
                            info!("{:?}", header.get_value());

                            for part in mail.subparts.iter() {
                                info!("{:?}", part.headers);

                                let re = Regex::new(r#"(?:name|filename)="([^"]+)"#).unwrap();

                                let header_name = part.headers.iter().find_map(|h| {
                                    let values = h.get_value();

                                    if let Some(caps) = re.captures(&values) {
                                        if let Some(filename) = caps.get(1) {
                                            return Some(filename.as_str().to_string());
                                        }
                                    };

                                    None
                                });

                                match part.get_body_encoded() {
                                    Body::Base64(body) => {
                                        let data = body.get_decoded().unwrap();

                                        if let Some(filename) = header_name {
                                            let mut path = PathBuf::from("tmp/attachments/");

                                            if let Ok(is_exist) = fs::try_exists(&path).await {
                                                if !is_exist {
                                                    fs::create_dir_all("tmp/attachments").await.map_err(|e| MyError::CreateDirError {
                                                        path: path.to_str().unwrap().to_string(),
                                                        source: e,
                                                    })?
                                                }
                                            } else {
                                                info!("Can't find path");
                                            }

                                            let new_folders = format!("{}/{}", p.timestamp.to_string(), p.from.to_string());

                                            fs::create_dir_all(&path.join(new_folders)).await.map_err(|e| MyError::CreateDirError {
                                                path: path.to_str().unwrap().to_string(),
                                                source: e,
                                            })?;

                                            path.push(p.timestamp.to_string());
                                            path.push(p.from.to_string());
                                            path.push(filename.to_string());

                                            let mut file = File::create(path).await.map_err(|e| MyError::CreateFileError {
                                                filename: filename.clone(),
                                                source: e,
                                            })?;
                                            file.write_all(&data).await?;

                                            p.attachments.push(filename);
                                        } else {
                                            warn!("Got attachment without filename");
                                        }
                                    }
                                    Body::QuotedPrintable(html) => {
                                        if let Ok(html) = html.get_decoded_as_string() {
                                            p.body = Some(html.to_string());
                                        }
                                    }
                                    Body::SevenBit(text) => {
                                        if let Ok(text) = text.get_as_string() {
                                            p.message = Some(text);
                                        }
                                    }
                                    Body::EightBit(_) => {}
                                    Body::Binary(_) => {}
                                }
                            }
                        }
                        "Subject" => {
                            p.subject = header.get_value();
                        }
                        _ => error!("Unrecognized header: {}", header.get_key()),
                    }
                }

                let m = Mail {
                    subject: p.subject,
                    receivers: p
                        .to
                        .iter()
                        .map(|email| email.to_string())
                        .collect(),
                    sender: p.from.to_string(),
                    message: p.message.unwrap_or("".to_string()),
                    body: p.body.unwrap_or("".to_string()),
                    attachments: p.attachments,
                    timestamp: now.timestamp(),
                };

                let mail_request = MailRequest {
                    domain: self.domain.clone(),
                    mail: Some(m),
                };

                self.domain = "".to_owned();

                self.channel.send(mail_request).await.unwrap();
                info!("Send mail to channel");
            }
            Request::Quit => {}
        }

        Ok(())
    }
}
