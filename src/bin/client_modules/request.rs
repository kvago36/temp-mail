use mailparse::parse_mail;
use mailparse::{ParsedMail};

use mail::email::Email;
use mail::error::MyError;

#[derive(Debug)]
pub enum Request<'a> {
    Hello(String),
    Mail(Email),
    Recipient(Email),
    Data(ParsedMail<'a>),
    Quit,
}

impl<'a> Request<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, MyError> {
        match &bytes[0..4] {
            b"HELO" | b"EHLO" => {
                let s = std::str::from_utf8(&bytes[5..]).unwrap();
                Ok(Request::Hello(s.trim_end().to_owned()))
            }
            b"MAIL" => {
                let s = std::str::from_utf8(&bytes[10..]).unwrap();

                Email::new(s.trim_end()).map(|email| Request::Mail(email))
            }
            b"RCPT" => {
                let s = std::str::from_utf8(&bytes[8..]).unwrap();

                Email::new(s.trim_end()).map(|email| Request::Recipient(email))
            }
            b"DATA" => match parse_mail(&bytes) {
                Ok(parsed) => Ok(Request::Data(parsed)),
                Err(_) => Err(MyError::ParseError),
            },
            _ => Err(MyError::UnknownCommand),
        }
    }
}
