use reqwest::StatusCode;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Client error")]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    Rest(#[from] RestError),
}

/// An application-level error returned by the REST API.
#[derive(Debug, Clone, Error)]
pub struct RestError {
    status: StatusCode,
    message: Option<String>,
}

impl RestError {
    /// Create a new `RestError` with the specified status code and human-friendly message.
    ///
    /// # Panics
    /// This function will panic if `status` is not a 4xx or 5xx error code.
    pub fn new(status: StatusCode, message: Option<String>) -> Self {
        if !(status.is_client_error() || status.is_server_error()) {
            panic!(
                "RestError should only be constructed with 4xx or 5xx status code; got {}",
                status
            );
        }

        Self { status, message }
    }

    /// Get the status code associated with the REST error.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the message returned by the system, if one is available.
    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|s| s.as_str())
    }
}

impl fmt::Display for RestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.status)?;
        if let Some(reason) = self.status.canonical_reason() {
            write!(f, "/{}", reason)?;
        }

        if let Some(message) = &self.message {
            write!(f, ": {}", message)
        } else {
            write!(f, " (No message provided)")
        }
    }
}
