use std::fmt;
use std::convert::From;
use std::error::Error as StdError;

use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use serde::de::value::Error as ValueError;

mod http;

pub use self::http::{ErrorBody, HttpError};

/// An error encountered while performing an operation.
///
/// At the moment, this is fairly unstable. It is recommended to terminate and surface an error.
#[derive(Debug)]
pub struct Error {
    cause: Option<Box<StdError>>,
}

impl Error {
    /// Creates a new error object with the specified inner error.
    fn with_cause(cause: Box<StdError>) -> Self {
        Error { cause: Some(cause) }
    }
}

impl StdError for Error {
    fn description(&self) -> &'static str {
        "ExtraHop API Error"
    }

    fn cause(&self) -> Option<&StdError> {
        self.cause.as_ref().map(|v| v.as_ref())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.cause)
    }
}

/// TODO: Check specifically for SSL error due to cert issue, and surface specifically.
/// https://doc.rust-lang.org/std/error/trait.Error.html#method.downcast_ref-2
impl From<ReqwestError> for Error {
    fn from(v: ReqwestError) -> Self {
        Error::with_cause(Box::new(v))
    }
}

impl From<SerdeJsonError> for Error {
    fn from(v: SerdeJsonError) -> Self {
        Error::with_cause(Box::new(v))
    }
}

impl From<HttpError> for Error {
    fn from(v: HttpError) -> Self {
        Error::with_cause(Box::new(v))
    }
}

impl From<ValueError> for Error {
    fn from(v: ValueError) -> Self {
        Error::with_cause(Box::new(v))
    }
}