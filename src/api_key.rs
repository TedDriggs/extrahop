use std::convert::From;
use std::fmt;
use std::str::FromStr;

use reqwest::header::{self, Scheme};

use {Error, ErrorKind, Result};

static FULL_PREFIX: &'static str = "ExtraHop apikey=";

/// A REST API key which authenticates the caller to the ExtraHop.
///
/// Clients must pass an API key in the [`Authorization`] header of every request. The 
/// [`extrahop::Client`] type handles this automatically, or it can be done manually.
///
/// # Examples
/// Manually adding the header to a request:
///
/// ```rust,no_run
/// # extern crate reqwest;
/// # extern crate extrahop;
/// # fn main() {    
/// use reqwest::header::Authorization;
/// use extrahop::ApiKey;
///
/// let header = Authorization(ApiKey::new("your-key"));
/// let _req = reqwest::Client::new().get("https://extrahop.com/api/v1/devices").header(header).send();
/// # }
/// ```
///
/// As a best practice, API keys should be collected from the user at runtime, or stored in an 
/// environment variable or separate file; they should not be checked in with source code.
///
/// [`Authorization`]: https://docs.rs/reqwest/0.4.0/reqwest/header/struct.Authorization.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Creates a new `ApiKey`. This removes any leading or trailing whitespace.
    pub fn new<S: Into<String>>(key: S) -> Self {
        ApiKey(key.into().trim().to_string())
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

impl From<String> for ApiKey {
    fn from(v: String) -> Self {
        ApiKey::new(v.to_string())
    }
}

impl<'a> From<&'a str> for ApiKey {
    fn from(v: &str) -> Self {
        ApiKey::new(v.to_string())
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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with(FULL_PREFIX) {
            Ok(ApiKey::new(s.trim_left_matches(FULL_PREFIX)))
        } else {
            Err(ErrorKind::ApiKeyParseError.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    
    use ErrorKind;
    use super::ApiKey;
    
    #[test]
    fn parse_header() {
        let full_header = "ExtraHop apikey=1a2b3c4d5e";
        let parsed = full_header.parse().expect("Parsing should handle a well-formed header body");
        assert_eq!(ApiKey::new("1a2b3c4d5e"), parsed);
    }
    
    #[test]
    fn parse_error() {
        let bare_key = "1a2b3c4d5e";
        let parse_err = ApiKey::from_str(bare_key).unwrap_err();
        if let ErrorKind::ApiKeyParseError = *parse_err.kind() {
            println!("{}", parse_err);
        } else {
            panic!("Error should have been an API key parse failure");
        }
    }
}