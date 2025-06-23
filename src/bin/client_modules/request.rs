use log::info;
use mailparse::ParsedMail;
use regex::Regex;

use mail::email::Email;
use mail::error::MyError;

#[derive(Debug)]
pub enum Request<'a> {
    Hello(String),
    Mail(Email),
    Reset,
    Recipient(Email),
    Data,
    Payload(ParsedMail<'a>),
    Quit,
}

impl<'a> Request<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, MyError> {
        match &bytes[0..4] {
            b"HELO" | b"EHLO" | b"helo" | b"ehlo" => {
                let s = std::str::from_utf8(&bytes[5..]).unwrap();
                Ok(Request::Hello(s.trim_end().to_owned()))
            }
            b"RSET" | b"rset" => Ok(Request::Reset),
            b"MAIL" | b"mail" => {
                let re = Regex::new(r"<([^>]+)>").expect("Failed to compile regex");
                let s = std::str::from_utf8(&bytes[10..]).unwrap();

                info!("MAIL: {}", s);

                if let Some(caps) = re.captures(s) {
                    let email = &caps[1];

                    Email::new(email).map(|e| Request::Mail(e))
                } else {
                    Err(MyError::ParseError)
                }
            }
            b"RCPT" | b"rcpt" => {
                let re = Regex::new(r"<([^>]+)>").expect("Failed to compile regex");
                let s = std::str::from_utf8(&bytes[8..]).unwrap();

                info!("RCPT: {}", s);

                if let Some(caps) = re.captures(s) {
                    let email = &caps[1];

                    Email::new(email).map(|e| Request::Recipient(e))
                } else {
                    Err(MyError::ParseError)
                }
            }
            b"DATA" | b"data" => Ok(Request::Data),
            b"QUIT" | b"quit" => Ok(Request::Quit),
            _ => Err(MyError::UnknownCommand),
        }
    }
}
