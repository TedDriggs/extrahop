use crate::{Error, RestError, Result};
use async_trait::async_trait;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// An ExtraHop REST API response.
#[async_trait]
pub trait ApiResponse: Sized {
    /// Checks if the status code returned was in the 2xx range. If so,
    /// returns the underlying response for further processing; otherwise
    /// returns an error.
    async fn validate_status(self) -> Result<Response>;

    /// Checks if the status code returned is in the 2xx range, and if so
    /// attempts to deserialize the response body as JSON into `T`.
    async fn validate_and_read<T: DeserializeOwned>(self) -> Result<T>;
}

#[derive(Deserialize)]
struct ApiError {
    error_message: String,
}

#[async_trait]
impl ApiResponse for Response {
    async fn validate_status(mut self) -> Result<Response> {
        if !self.status().is_success() {
            Err(RestError::new(
                self.status(),
                self.json::<ApiError>().await.ok().map(|e| e.error_message),
            )
            .into())
        } else {
            Ok(self)
        }
    }

    async fn validate_and_read<T: DeserializeOwned>(self) -> Result<T> {
        self.validate_status()
            .await?
            .json::<T>()
            .await
            .map_err(Error::from)
    }
}
