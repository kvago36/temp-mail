use regex::Regex;

use crate::error::MyError;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(email: &str) -> Result<Self, MyError> {
        let re = Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
        if re.is_match(email) {
            Ok(Email(email.to_string()))
        } else {
            Err(MyError::ParseError)
        }
    }

    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap()
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
