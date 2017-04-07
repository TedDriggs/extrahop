use reqwest::{self, Method, RequestBuilder};

use ApiKey;

/// A client for making authenticated requests to the ExtraHop REST API.
///
/// The client encapsulates the host name and API key needed to make calls to
/// the appliance.
pub struct Client {
    host: String,
    api_key: ApiKey,
    r_client: reqwest::Client,
}

impl Client {
    /// Creates a new client which connects to the specified host using the provided key.
    pub fn new<IS: Into<String>>(host: IS, api_key: ApiKey) -> Self {
        Client {
            host: host.into(),
            api_key: api_key,
            r_client: reqwest::Client::new().expect("reqwest::Client creation failure not handled"),
        }
    }

    /// Gets the appliance's host string.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Creates a GET request builder for the provided relative path with the 
    /// `Authorization` header included.
    ///
    /// The path should not include the `/api/v1` prefix.
    ///
    /// ```rust,no_run
    /// # use extrahop::ApiKey;
    /// let client = Client::new("extrahop.i.northwind.com", ApiKey::new("key".to_string()));
    /// client.get("/whitelist/devices").send().unwrap();
    /// ```
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::Get, path)
    }

    /// Creates a POST request builder for the provided relative path with the 
    /// `Authorization` header included.
    ///
    /// The path should not include the `/api/v1` prefix.
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.request(Method::Post, path)
    }
    
    /// Creates a PATCH request builder for the provided relative path with the 
    /// `Authorization` header included.
    ///
    /// The path should not include the `/api/v1` prefix.
    pub fn patch(&self, path: &str) -> RequestBuilder {
        self.request(Method::Patch, path)
    }
    
    /// Creates a PUT request builder for the provided relative path with the 
    /// `Authorization` header included.
    ///
    /// The path should not include the `/api/v1` prefix.
    pub fn put(&self, path: &str) -> RequestBuilder {
        self.request(Method::Put, path)
    }
    
    /// Creates a DELETE request builder for the provided relative path with the 
    /// `Authorization` header included.
    ///
    /// The path should not include the `/api/v1/` prefix.
    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.request(Method::Delete, path)
    }

    /// Creates a request builder for the provided relative path with the 
    /// `Authorization` header included.
    ///
    /// The path should not include the `/api/v1` prefix.
    ///
    /// ```rust,no_run
    /// # extern crate reqwest;
    /// # use extrahop::ApiKey;
    /// use reqwest::Method;
    /// let client = Client::new("extrahop", ApiKey::new("key".to_string()));
    /// client.request(Method::Get, "/whitelist/devices").send().unwrap();
    /// ```
    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let leading_slash = if path.starts_with("/") { "" } else { "/" };
        
        self.r_client
            .request(method, &format!("https://{}/api/v1/{}{}", self.host, leading_slash, path))
            .header(self.api_key.clone().to_header())
    }
}