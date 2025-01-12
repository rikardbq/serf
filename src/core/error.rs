use core::str;
use std::fmt;

use serde::Serialize;

pub const UNDEFINED: &str = "Undefined server error";
pub const DATABASE_NOT_EXIST: &str = "Database doesn't exist";
pub const USER_NOT_EXIST: &str = "User doesn't exist";
pub const USER_NOT_ALLOWED: &str = "User privilege too low";
pub const HEADER_MISSING: &str = "Request is missing a required header";
pub const HEADER_MALFORMED: &str = "Request header value is malformed";
pub const RESOURCE_NOT_EXIST: &str = "Resource doesn't exist";

#[derive(Debug, PartialEq, Serialize)]
pub enum ErrorKind {
    Undefined,
    DatabaseNotExist,
    UserNotExist,
    UserNotAllowed,
    HeaderMissing,
    HeaderMalformed,
    ResourceNotExist,
}

pub trait SerfError<'a> {
    fn default() -> Error;
    fn with_message(message: &'a str) -> Error;
}

#[derive(Debug, Serialize)]
pub struct Error {
    pub message: String,
    pub source: ErrorKind,
}

impl<'a> Error {
    pub fn new(message: &'a str, kind: ErrorKind) -> Self {
        Error {
            message: message.to_string(),
            source: kind,
        }
    }
}

pub struct UndefinedError;
impl<'a> SerfError<'a> for UndefinedError {
    fn default() -> Error {
        Error::new(UNDEFINED, ErrorKind::Undefined)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::Undefined)
    }
}

pub struct DatabaseNotExistError;
impl<'a> SerfError<'a> for DatabaseNotExistError {
    fn default() -> Error {
        Error::new(DATABASE_NOT_EXIST, ErrorKind::DatabaseNotExist)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::DatabaseNotExist)
    }
}

pub struct UserNotExistError;
impl<'a> SerfError<'a> for UserNotExistError {
    fn default() -> Error {
        Error::new(USER_NOT_EXIST, ErrorKind::UserNotExist)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::UserNotExist)
    }
}

pub struct UserNotAllowedError;
impl<'a> SerfError<'a> for UserNotAllowedError {
    fn default() -> Error {
        Error::new(USER_NOT_ALLOWED, ErrorKind::UserNotAllowed)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::UserNotAllowed)
    }
}

pub struct HeaderMissingError;
impl<'a> SerfError<'a> for HeaderMissingError {
    fn default() -> Error {
        Error::new(HEADER_MISSING, ErrorKind::HeaderMissing)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::HeaderMissing)
    }
}

pub struct HeaderMalformedError;
impl<'a> SerfError<'a> for HeaderMalformedError {
    fn default() -> Error {
        Error::new(HEADER_MALFORMED, ErrorKind::HeaderMalformed)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::HeaderMalformed)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ErrorKind {}

impl<'a> std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
