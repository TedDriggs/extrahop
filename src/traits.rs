use std::result::Result as StdResult;

use reqwest::Error as ReqwestError;
use reqwest::Response;
use serde::{de, Serialize};

use {Error, Result};
use errors::RestError;

/// Marker trait for types that are meant to use in PATCH requests.
pub trait Patch: Default + Serialize {}

/// Validator and deserializer for REST API responses.
pub trait ApiResponse : Sized {
    /// Checks if the status code returned was in the 2xx range. If so,
    /// returns the underlying response for further processing; otherwise
    /// returns an error.
    fn validate_status(self) -> Result<Response>;
    
    /// Attempts to parse the response as JSON into `T`.
    fn deserialize<T: de::DeserializeOwned>(self) -> Result<T>;
    
    /// Validates the status code and attempts to deserialize into `T` in one step.
    fn validate_and_read<T: de::DeserializeOwned>(self) -> Result<T> {
        self.validate_status()?.deserialize()
    }
}

impl ApiResponse for Response {
    fn validate_status(mut self) -> Result<Response> {
        if !self.status().is_success() {
            Err(RestError::new(self.status().clone(), self.json().ok()).into())
        } else {
            Ok(self)
        }
    }
    
    fn deserialize<T: de::DeserializeOwned>(self) -> Result<T> {
        self.validate_status()?.json().map_err(Error::from)
    }
}

impl ApiResponse for StdResult<Response, ReqwestError> {
    fn validate_status(self) -> Result<Response> {
        self.map_err(Error::from).and_then(ApiResponse::validate_status)
    }
    
    fn deserialize<T: de::DeserializeOwned>(self) -> Result<T> {
        self.validate_status().and_then(ApiResponse::deserialize)
    }
}