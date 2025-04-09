use core::str;
use std::fmt;

use super::serf_proto::{Error, ErrorKind};

pub const UNDEFINED: &str = "Undefined server error";
pub const DATABASE: &str = "Database responded with error";
pub const USER_NOT_EXIST: &str = "User doesn't exist";
pub const USER_NOT_ALLOWED: &str = "User privilege too low";
pub const HEADER_MISSING: &str = "Request is missing a required header";
pub const HEADER_MALFORMED: &str = "Request header value is malformed";
pub const RESOURCE_NOT_EXIST: &str = "Resource doesn't exist";
pub const PROTOPACKAGE: &str = "Proto package verification or signing error";

pub trait SerfError<'a> {
    fn default() -> Error;
    fn with_message(message: &'a str) -> Error;
}

pub struct UndefinedError;
pub struct DatabaseError;
pub struct UserNotExistError;
pub struct UserNotAllowedError;
pub struct HeaderMissingError;
pub struct HeaderMalformedError;
pub struct ResourceNotExistError;
pub struct ProtoPackageError;

impl Error {
    pub fn new(message: &str, kind: ErrorKind) -> Self {
        Error {
            message: message.to_string(),
            source: kind.into(),
        }
    }
}

impl<'a> SerfError<'a> for UndefinedError {
    fn default() -> Error {
        Error::new(UNDEFINED, ErrorKind::Undefined)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::Undefined)
    }
}

impl<'a> SerfError<'a> for DatabaseError {
    fn default() -> Error {
        Error::new(DATABASE, ErrorKind::Database)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::Database)
    }
}

impl<'a> SerfError<'a> for UserNotExistError {
    fn default() -> Error {
        Error::new(USER_NOT_EXIST, ErrorKind::UserNotExist)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::UserNotExist)
    }
}

impl<'a> SerfError<'a> for UserNotAllowedError {
    fn default() -> Error {
        Error::new(USER_NOT_ALLOWED, ErrorKind::UserNotAllowed)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::UserNotAllowed)
    }
}

impl<'a> SerfError<'a> for HeaderMissingError {
    fn default() -> Error {
        Error::new(HEADER_MISSING, ErrorKind::HeaderMissing)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::HeaderMissing)
    }
}

impl<'a> SerfError<'a> for HeaderMalformedError {
    fn default() -> Error {
        Error::new(HEADER_MALFORMED, ErrorKind::HeaderMalformed)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::HeaderMalformed)
    }
}

impl<'a> SerfError<'a> for ResourceNotExistError {
    fn default() -> Error {
        Error::new(RESOURCE_NOT_EXIST, ErrorKind::ResourceNotExist)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::ResourceNotExist)
    }
}

impl<'a> SerfError<'a> for ProtoPackageError {
    fn default() -> Error {
        Error::new(PROTOPACKAGE, ErrorKind::ProtoPackage)
    }

    fn with_message(message: &'a str) -> Error {
        Error::new(message, ErrorKind::ProtoPackage)
    }
}

impl ProtoPackageError {
    pub fn signing_error(message: &str) -> Error {
        ProtoPackageError::with_message(&format!("{}: {}", "SIGN", message))
    }
    pub fn verification_error(message: &str) -> Error {
        ProtoPackageError::with_message(&format!("{}: {}", "VERIFY", message))
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// may need to dig a little more on how to fix this shit if its even needed?
// impl std::error::Error for ErrorKind {}

// impl std::error::Error for Error {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         Some(&self.source())
//     }

//     fn cause(&self) -> Option<&dyn std::error::Error> {
//         self.source()
//     }
// }
