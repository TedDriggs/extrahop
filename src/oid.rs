use serde::{Deserialize, Serialize};

/// A metric source ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Oid(u64);

impl Oid {
    /// Creates a new OID.
    pub fn new(id: u64) -> Self {
        Oid(id)
    }
    /// Gets a representation of the OID for use in a URL path.
    pub fn as_url_part(&self) -> String {
        format!("{}", self.0)
    }
}

impl From<u64> for Oid {
    fn from(val: u64) -> Self {
        Oid::new(val)
    }
}
