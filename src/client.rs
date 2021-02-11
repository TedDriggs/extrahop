//! Clients for calling the ExtraHop REST API, supporting both Reveal(x) 360 and direct appliance
//! connections.

use reqwest::{header, Certificate, Method, RequestBuilder};
use secstr::SecUtf8;
use serde::Deserialize;
use std::{
    cell::RefCell,
    fmt,
    time::{Duration, Instant},
};
use thiserror::Error;
use url::{ParseError, Url};

/// Add convenience methods for the major HTTP methods. These depend on the presence of
/// a `request` method for the struct in whose impl block these are placed.
macro_rules! methods {
    () => {
        /// Make a `GET` request to the specified endpoint.
        ///
        /// # Example
        /// ```rust,ignore
        /// client.get("v1/devices")
        /// ```
        pub fn get(&self, endpoint: &str) -> Result<RequestBuilder, ParseError> {
            self.request(Method::GET, endpoint)
        }

        /// Make a `POST` request to the specified endpoint.
        ///
        /// # Example
        /// ```rust,ignore
        /// client.post("v1/records/search")
        /// ```
        pub fn post(&self, endpoint: &str) -> Result<RequestBuilder, ParseError> {
            self.request(Method::POST, endpoint)
        }

        /// Make a `PUT` request to the specified endpoint.
        ///
        /// # Example
        /// ```rust,ignore
        /// client.put("v1/analysispriority/config/2")
        /// ```
        pub fn put(&self, endpoint: &str) -> Result<RequestBuilder, ParseError> {
            self.request(Method::PUT, endpoint)
        }

        /// Make a `PUT` request to the specified endpoint.
        ///
        /// # Example
        /// ```rust,ignore
        /// client.patch("v1/detections/22")
        /// ```
        pub fn patch(&self, endpoint: &str) -> Result<RequestBuilder, ParseError> {
            self.request(Method::PATCH, endpoint)
        }

        /// Make a `PUT` request to the specified endpoint.
        ///
        /// # Example
        /// ```rust,ignore
        /// client.delete("v1/triggers/19")
        /// ```
        pub fn delete(&self, endpoint: &str) -> Result<RequestBuilder, ParseError> {
            self.request(Method::DELETE, endpoint)
        }
    };
}

#[derive(Deserialize)]
struct SaasCredentialResponse {
    access_token: SecUtf8,
}

/// An error while connecting to - or refreshing credentials with - a Reveal(x) 360 tenant.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SaasConnectError {
    #[error("Host is not a valid domain")]
    InvalidDomain(#[from] url::ParseError),
    #[error("Unable to get access token")]
    Reqwest(#[from] reqwest::Error),
}

struct SaasAccessToken {
    access_token: SecUtf8,
    start: Instant,
}

impl SaasAccessToken {
    /// Whether the access token will expire before the specified duration has passed.
    pub fn expires_in_next(&self, duration: Duration) -> bool {
        self.start.elapsed() + duration >= Duration::from_secs(3600)
    }

    /// Get access to the token value.
    fn unsecure(&self) -> &str {
        self.access_token.unsecure()
    }
}

/// A client for making requests of a Reveal(x) 360 tenant, e.g. `example.cloud.extrahop.com`.
pub struct Saas {
    root: Url,
    id: String,
    /// The API credential's secret; used to request an access token.
    secret: SecUtf8,
    /// The temporary access token used in all API calls.
    access_token: RefCell<SaasAccessToken>,
    client: reqwest::Client,
}

impl Saas {
    /// Create a new API client for communicating with a Reveal(x) 360 tenant.
    ///
    /// Reveal(x) 360 requires API calls be authorized using a temporary access token. Creating a client
    /// handles the initial token generation from the API credential's ID and secret; this will be good for
    /// one hour.
    ///
    /// The domain should be the fully-qualified domain name, e.g. `example.cloud.extrahop.com`.
    pub async fn new(domain: &str, id: String, secret: SecUtf8) -> Result<Self, SaasConnectError> {
        let mut root =
            Url::parse("https://temp.cloud.extrahop.com").expect("Hardcoded starting URL is valid");
        root.set_host(Some(domain))?;

        let client = reqwest::Client::new();
        let access_token = Saas::get_access_token(&client, &root, &id, &secret).await?;

        Ok(Self {
            root,
            id,
            secret,
            access_token: RefCell::new(access_token),
            client,
        })
    }

    methods!();

    /// Make a request to the specified endpoint using the specified method.
    ///
    /// # Example
    /// ```rust,ignore
    /// client.request(Method::POST, "v1/records/search")
    /// ```
    pub fn request(&self, method: Method, url: &str) -> Result<RequestBuilder, ParseError> {
        Ok(self
            .client
            .request(method, self.root.join("api")?.join(url)?)
            .bearer_auth(self.access_token.borrow().unsecure()))
    }

    /// Generate a new access token and replace the one currently in use.
    pub async fn renew_access_token(&self) -> Result<(), SaasConnectError> {
        let new_access_token =
            Saas::get_access_token(&self.client, &self.root, &self.id, &self.secret).await?;
        self.access_token.replace(new_access_token);
        Ok(())
    }

    async fn get_access_token(
        client: &reqwest::Client,
        host: &Url,
        id: &str,
        secret: &SecUtf8,
    ) -> Result<SaasAccessToken, SaasConnectError> {
        // Capture the session start time before it happens, so the server doesn't expire our
        // temporary key before the client thinks it expires.
        let start = Instant::now();

        let response = client
            .post(host.join("oauth2/token").expect("OAuth2 path is valid"))
            .basic_auth(&id, Some(secret.unsecure()))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?;

        let SaasCredentialResponse { access_token } = response.json().await?;

        Ok(SaasAccessToken {
            access_token,
            start,
        })
    }
}

impl fmt::Display for Saas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (SaaS)", self.root.host_str().unwrap_or("NONE"))
    }
}

/// Error encountered while connecting to a specific appliance.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApplianceClientError {
    #[error("Invalid host")]
    InvalidHost(#[from] ParseError),
    #[error("Unable to initialize client")]
    Reqwest(#[from] reqwest::Error),
}

/// Appliance client certificate validation behavior.
pub enum CertVerification {
    /// Use the system's standard certificate validation rules and root certificates.
    System,
    /// Accept any certificate. This is dangerous and should not be done lightly.
    DangerAcceptInvalid,
    /// Add the specified certificate as a root certificate for this client. This allows
    /// the safe use of self-signed appliance certs.
    Custom(Certificate),
}

impl Default for CertVerification {
    fn default() -> Self {
        Self::System
    }
}

/// A client to communicate with a specific ExtraHop appliance.
pub struct Appliance {
    root: Url,
    api_key: SecUtf8,
    client: reqwest::Client,
}

impl Appliance {
    /// Create a new client for communicating with a specific ExtraHop appliance.
    pub fn new(
        host: &str,
        api_key: SecUtf8,
        cert_verification: CertVerification,
    ) -> Result<Self, ApplianceClientError> {
        let mut root = Url::parse("https://temporary/").expect("Hardcoded URL is valid");
        root.set_host(Some(host))?;
        let client = match cert_verification {
            CertVerification::System => reqwest::Client::new(),
            CertVerification::DangerAcceptInvalid => reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?,
            CertVerification::Custom(cert) => reqwest::Client::builder()
                .add_root_certificate(cert)
                .build()?,
        };

        Ok(Self {
            root,
            api_key,
            client,
        })
    }

    methods!();

    /// Make a request to the specified endpoint using the specified method.
    ///
    /// # Example
    /// ```rust,ignore
    /// client.request(Method::POST, "v1/records/search")
    /// ```
    pub fn request(&self, method: Method, url: &str) -> Result<RequestBuilder, ParseError> {
        Ok(self
            .client
            .request(method, self.root.join("api")?.join(url)?)
            .header(
                header::AUTHORIZATION,
                format!("ExtraHop apikey={}", self.api_key),
            ))
    }
}

impl fmt::Display for Appliance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (Appliance)", self.root.host_str().unwrap_or("NONE"))
    }
}

/// Concrete client implementation. This is not public to avoid callers from relying on
/// its internal structure.
enum Inner {
    Saas(Saas),
    Appliance(Appliance),
}

/// A client to connect to either the Reveal(x) 360 SaaS REST API or to a specific appliance.
///
/// There is strong overlap between the SaaS and EDA/ECA APIs, so most code written for one
/// will work unmodified for the other. Functions can accept `Client` to be agnostic to which
/// implementation they communicate with.
pub struct Client {
    inner: Inner,
}

impl Client {
    /// Create a new client for a Reveal(x) 360 tenant.
    ///
    /// The domain should be the fully-qualified domain name, e.g. `example.cloud.extrahop.com`.
    pub async fn new_saas(
        domain: &str,
        id: String,
        secret: SecUtf8,
    ) -> Result<Self, SaasConnectError> {
        Ok(Saas::new(domain, id, secret).await?.into())
    }

    /// Create a new client for a specific appliance.
    pub async fn new_appliance(
        host: &str,
        api_key: SecUtf8,
        certs: CertVerification,
    ) -> Result<Self, ApplianceClientError> {
        Ok(Appliance::new(host, api_key, certs)?.into())
    }

    /// Check if the client is talking to a Reveal(x) 360 tenant.
    pub fn is_saas(&self) -> bool {
        if let Inner::Saas(_) = self.inner {
            true
        } else {
            false
        }
    }

    /// Check if the client is talking to a specific ExtraHop appliance.
    pub fn is_appliance(&self) -> bool {
        !self.is_saas()
    }

    methods!();

    /// Make a request to the specified endpoint using the specified method.
    ///
    /// # Example
    /// ```rust,ignore
    /// client.request(Method::POST, "v1/records/search")
    /// ```
    pub fn request(&self, method: Method, endpoint: &str) -> Result<RequestBuilder, ParseError> {
        match &self.inner {
            Inner::Saas(client) => client.request(method, endpoint),
            Inner::Appliance(client) => client.request(method, endpoint),
        }
    }

    /// Ensure the client will continue to be able to make API requests.
    ///
    /// For appliance clients, this is a no-op. For SaaS clients, this will generate
    /// a new access token if the current token is approaching expiration.
    pub async fn maintain_access(&self) -> Result<(), SaasConnectError> {
        if let Inner::Saas(client) = &self.inner {
            if client
                .access_token
                .borrow()
                .expires_in_next(Duration::from_secs(300))
            {
                return client.renew_access_token().await;
            }
        }

        Ok(())
    }
}

impl From<Appliance> for Client {
    fn from(client: Appliance) -> Self {
        Self {
            inner: Inner::Appliance(client),
        }
    }
}

impl From<Saas> for Client {
    fn from(client: Saas) -> Self {
        Self {
            inner: Inner::Saas(client),
        }
    }
}
