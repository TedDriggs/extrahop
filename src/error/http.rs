use std::fmt;
use std::error::Error as StdError;
use reqwest::{Response, StatusCode};

/// A 4xx or 5xx response from the ExtraHop appliance.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpError {
    pub status: StatusCode,
    pub body: Option<ErrorBody>,
}

impl HttpError {
    /// Inspect an HTTP response to see if the call succeeded, and either return
    /// the original response unaltered or return an instance of this type.
    pub fn inspect(mut rsp: Response) -> Result<Response, HttpError> {
        if !rsp.status().is_success() {
            Err(HttpError {
                status: rsp.status().clone(),
                body: rsp.json().ok()
            })
        } else {
            Ok(rsp)
        }
    }
}

impl StdError for HttpError {
    fn description(&self) -> &str {
        "ExtraHop appliance returned an error"
    }
    
    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {:?}", self.status, self.body)
    }
}

/// The body of an HTTP error response from the appliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    error_message: String,
}

