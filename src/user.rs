use serde::{Deserialize, Serialize};

/// Canonical ID for an appliance user.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    /// Creates and canonicalizes a username.
    pub fn new<IS: Into<String>>(name: IS) -> Self {
        Username(name.into().trim().into())
    }
    /// Gets the username as a JSON-suitable string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
