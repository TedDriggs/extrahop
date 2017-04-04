use std::fmt;
use std::error::Error as StdError;
use reqwest::StatusCode;

use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use serde::de::value::Error as ValueError;

error_chain! {
    foreign_links {
        Http(ReqwestError);
        
        Json(SerdeJsonError);
        
        Parse(ValueError);
        
        Rest(RestError);
    }
}

/// A 4xx or 5xx response from the ExtraHop appliance.
#[derive(Debug, Clone, PartialEq)]
pub struct RestError {
    status: StatusCode,
    body: Option<ErrorBody>,
}

impl RestError {
    /// Create a new REST API error.
    pub fn new<B: Into<Option<ErrorBody>>>(status: StatusCode, body: B) -> Self {
        RestError {
            status: status,
            body: body.into(),
        }
    }
    
    /// Gets the HTTP status code of the error.
    pub fn status(&self) -> StatusCode {
        self.status
    }
    
    /// Gets the body of the error, if one was provided in well-formed JSON.
    pub fn body(&self) -> Option<&ErrorBody> {
        self.body.as_ref()
    }
}

impl StdError for RestError {
    fn description(&self) -> &str {
        "ExtraHop appliance returned an error"
    }
    
    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl fmt::Display for RestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {:?}", self.status, self.body)
    }
}

/// The body of an HTTP error response from the appliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    error_message: String,
}

impl ErrorBody {
    /// Gets the human-friendly message returned by the appliance.
    pub fn message(&self) -> &str {
        &self.error_message
    }
}