use std::convert::From;
use std::fmt;
use std::str::FromStr;

use reqwest::header::{self, Scheme};


static FULL_PREFIX: &'static str = "ExtraHop apikey=";

/// A REST API key which authenticates the caller to the ExtraHop.
///
/// Clients must pass an API key in the [`Authorization`] header of every request.
///
/// ```rust
/// # extern crate reqwest;
/// # extern crate extrahop;
/// # fn main() {
/// use reqwest::header::Authorization;
/// use extrahop::ApiKey;
///
/// let header = Authorization(ApiKey::new("your-key".to_string()));
/// // insert into request using "header()" method.
/// # }
/// ```
///
/// As a best practice, API keys should be collected from the user at runtime or stored in a separate
/// file; they should not be checked in with source code.
///
/// [`Authorization`]: https://docs.rs/reqwest/0.4.0/reqwest/header/struct.Authorization.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Creates a new `ApiKey`. This removes any leading or trailing whitespace.
    pub fn new(key: String) -> Self {
        ApiKey(key.trim().to_string())
    }

    /// Converts the API key to an Authorization header.
    pub fn to_header(self) -> header::Authorization<ApiKey> {
        header::Authorization(self)
    }
}

impl Scheme for ApiKey {
    fn scheme() -> Option<&'static str> {
        // The prefix is "ExtraHop"
        Some(&FULL_PREFIX[..8])
    }

    fn fmt_scheme(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The prefix is "apikey="
        write!(f, "{}{}", &FULL_PREFIX[9..], self.0)
    }
}

impl From<ApiKey> for header::Authorization<ApiKey> {
    fn from(v: ApiKey) -> Self {
        v.to_header()
    }
}

/// Parses a full HTTP header; required by `reqwest::header::Scheme`.
///
/// This should not be used for converting a regular string into an API key; for
/// that, use [`ApiKey::new`] instead.
///
/// [`ApiKey::new`]: #method.new
impl FromStr for ApiKey {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(FULL_PREFIX) {
            Ok(ApiKey::new(s.trim_left_matches(FULL_PREFIX).into()))
        } else {
            Err("ApiKey::from_str requires being passed the full value of an Authorization header")
        }
    }
}