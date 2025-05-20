use mailparse::{MailHeader, ParsedMail};
use tokio::sync::mpsc::Sender;

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
            recipients: Vec::new(),
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
                let mut subject = String::new();

                for header in mail.headers {
                    match header.get_key().as_str() {
                        "From" => {
                            let from = Email::new("test@test.com").unwrap();
                            self.sender = Some(from);
                        }
                        "Subject" => {
                            subject = header.get_value();
                        }
                        _ => println!("Unrecognized subject header: {}", header.get_key()),
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
                println!("Send mail to channel");
            }
            Request::Quit => {}
        }
    }
}
