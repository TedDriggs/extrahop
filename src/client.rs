use crate::{Error, Result};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::{Method, RequestBuilder};
use url::Url;

/// Builder to construct a `Client` with options set.
pub struct ClientBuilder {
    host: String,
    api_key: String,
    disable_certs: Option<bool>,
}

impl ClientBuilder {
    /// Create a new builder.
    ///
    /// Validation of the host and API key is deferred until `build` is called, so this function
    /// cannot fail.
    pub fn new(host: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            api_key: api_key.into(),
            disable_certs: None,
        }
    }

    /// Turn off certificate validation.
    pub fn dangerous_disable_cert_verification(mut self, disable: bool) -> Self {
        self.disable_certs = Some(disable);
        self
    }

    pub fn build(self) -> Result<Client> {
        let mut headers = HeaderMap::new();
        headers.append(
            AUTHORIZATION,
            format!("ExtraHop apikey={}", self.api_key)
                .parse()
                .map_err(|_| Error::ApiKey)?,
        );

        Ok(Client {
            base: format!("https://{}/api/", self.host).parse().unwrap(),
            inner: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .danger_accept_invalid_certs(self.disable_certs.unwrap_or_default())
                .build()?,
        })
    }
}

/// An asynchronous client to make requests to an ExtraHop system.
///
/// The client is linked to a specific ExtraHop API instance, identified by hostname.
///
/// # Paths
/// Include the version in the URL when making requests, e.g. `v1/devices`. The client
/// automatically adds the hostname and base.
pub struct Client {
    base: Url,
    inner: reqwest::Client,
}

impl Client {
    /// Create a new client for the specified host and API key with default settings.
    ///
    /// # Errors
    /// This function returns an error if the hostname is invalid or the API key is the empty string.
    pub fn new(host: impl Into<String>, api_key: impl Into<String>) -> Result<Self> {
        ClientBuilder::new(host, api_key).build()
    }

    /// Create a new `ClientBuilder`.
    pub fn builder(host: impl Into<String>, api_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(host, api_key)
    }

    /// Get the host to which this `Client` is associated.
    pub fn host(&self) -> &str {
        self.base.host_str().expect("The base URL must have a host")
    }

    pub fn get(&self, path: impl AsRef<str>) -> Result<RequestBuilder> {
        self.request(Method::GET, path)
    }

    pub fn post(&self, path: impl AsRef<str>) -> Result<RequestBuilder> {
        self.request(Method::POST, path)
    }

    pub fn put(&self, path: impl AsRef<str>) -> Result<RequestBuilder> {
        self.request(Method::PUT, path)
    }

    pub fn patch(&self, path: impl AsRef<str>) -> Result<RequestBuilder> {
        self.request(Method::PATCH, path)
    }

    pub fn delete(&self, path: impl AsRef<str>) -> Result<RequestBuilder> {
        self.request(Method::DELETE, path)
    }

    /// Create a request to send to the ExtraHop.
    ///
    /// # Example
    /// ```rust,norun
    /// use extrahop::Client;
    /// use reqwest::Method;
    ///
    /// Client::new("extrahop", "123").unwrap().request(Method::GET, "v1/devices");
    /// ```
    pub fn request(&self, method: Method, path: impl AsRef<str>) -> Result<RequestBuilder> {
        Ok(self.inner.request(
            method,
            self.base.join(path.as_ref().trim_start_matches('/'))?,
        ))
    }
}
