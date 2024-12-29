#[derive(PartialEq, Debug)]
pub enum ErrorReason {
    UserNotAllowed,
    UserNoDatabaseAccess,
    UnknownUser,
    InvalidSubject,
    InvalidIssuer,
    ResourceNotFound,
    MissingHeader,
}

pub const ERROR_USER_NOT_ALLOWED: &str = "ERROR=UserNotAllowed";
pub const ERROR_USER_NO_DATABASE_ACCESS: &str = "ERROR=UserNoDatabaseAccess";
pub const ERROR_UNKNOWN_USER: &str = "ERROR=UnknownUser";
pub const ERROR_INVALID_SUBJECT: &str = "ERROR=InvalidSubject";
pub const ERROR_INVALID_ISSUER: &str = "ERROR=InvalidIssuer";
pub const ERROR_DATABASE_NOT_FOUND: &str = "ERROR=DatabaseNotFound";
pub const ERROR_MISSING_HEADER: &str = "ERROR=MissingHeader";
pub const ERROR_MALFORMED_HEADER: &str = "ERROR=MalformedHeader";
pub const ERROR_FORBIDDEN: &str = "ERROR=Forbidden";
pub const ERROR_NOT_ACCEPTABLE: &str = "ERROR=NotAcceptable";
pub const ERROR_UNSPECIFIED: &str = "ERROR=UnspecifiedServerError";
