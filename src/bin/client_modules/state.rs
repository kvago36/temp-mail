use log::{debug, error, info};
use mailparse::body::Body;
use mailparse::{MailHeader, ParsedMail};
use regex::Regex;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use chrono::{DateTime, Local};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;

use crate::mail_proto::{Mail, MailRequest};

use mail::email::Email;
use mail::error::MyError;
use crate::client_modules::request::Request;

pub struct Payload {
    subject: String,
    from: Email,
    to: Vec<Email>,
    attachments: Vec<String>,
    body: Option<String>,
    message: Option<String>,
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
                // info!("Received data: {:?}", mail);
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
                };

                // let mut subject = String::new();

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
                                // let headers = HashMap::<String, String>::new();

                                let mut name = String::new();

                                for header in part.headers.iter() {
                                    let values = header.get_value();
                                    let re = Regex::new(r#"(?:name|filename)="([^"]+)"#).unwrap();

                                    if let Some(caps) = re.captures(&values) {
                                        if let Some(filename) = caps.get(1) {
                                            name = filename.as_str().to_string();
                                        }
                                    }
                                }

                                match part.get_body_encoded() {
                                    Body::Base64(body) => {
                                        let data = body.get_decoded().unwrap();

                                        // TODO: fix if name changes
                                        let mut file = File::create(name.clone()).await?;
                                        file.write_all(&data).await?;

                                        p.attachments.push(name);
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

                let now: DateTime<Local> = Local::now();

                let m = Mail {
                    subject: p.subject,
                    receivers: self
                        .recipients
                        .iter()
                        .map(|email| email.to_string())
                        .collect(),
                    sender: self.sender.take().expect("Should have sender!").to_string(),
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
