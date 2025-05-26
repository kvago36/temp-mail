use std::collections::HashMap;
use mailparse::{MailHeader, ParsedMail};
use mailparse::body::Body;
use tokio::sync::mpsc::Sender;
use tokio::fs::File;
use std::io::{Cursor, Read};
use log::{info, error, debug};
use tokio::io::AsyncWriteExt;
use regex::Regex;

use crate::mail_proto::{Mail, MailRequest};

use mail::email::Email;

use crate::client_modules::request::Request;

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

    pub(crate) async fn handle_request(&mut self, message: Request<'_>) {
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

                let mut subject = String::new();

                for header in mail.headers {
                    match header.get_key().as_str() {
                        "From" => {
                            let from = Email::new("test@test.com").unwrap();
                            self.sender = Some(from);
                        },
                        "To" => {
                            // same recipient command
                        },
                        "Content-Type" => {
                            info!("{:?}", header.get_value());

                            for part in mail.subparts.iter() {
                                info!("{:?}", part.headers);
                                // let headers = HashMap::<String, String>::new();

                                let mut name = String::new();;

                                for header in part.headers.iter() {
                                    let values = header.get_value();
                                    let re = Regex::new(r#"(?:name|filename)="([^"]+)"#).unwrap();

                                    if let Some(caps) = re.captures(&values) {
                                        if let Some(filename) = caps.get(1) {
                                            name = filename.as_str().to_string();
                                        }
                                    }
                                }

                                if let Body::Base64(body) = part.get_body_encoded() {
                                    let data = body.get_decoded().unwrap();

                                    let mut file = File::create(name).await.unwrap();
                                    file.write_all(&data).await.unwrap();

                                    // let base64 = base64_simd::STANDARD;
                                    // let encoded = base64.encode_to_string(data);

                                    // info!("{}", encoded);
                                }
                            }
                        },
                        "Subject" => {
                            subject = header.get_value();
                        },
                        _ => error!("Unrecognized header: {}", header.get_key()),
                    }
                }

                let m = Mail {
                    subject,
                    receivers: self
                        .recipients
                        .iter()
                        .map(|email| email.to_string())
                        .collect(),
                    sender: self.sender.take().expect("Should have sender!").to_string(),
                    message: "".to_string(),
                    timestamp: 0,
                };

                let mail_request = MailRequest {
                    domain: self.domain.clone(),
                    mail: Some(m),
                };

                self.recipients = Vec::new();
                self.domain = "".to_owned();

                self.channel.send(mail_request).await.unwrap();
                info!("Send mail to channel");
            }
            Request::Quit => {}
        }
    }
}
