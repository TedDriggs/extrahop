use serde::{Deserialize, Serialize};

/// Canonical ID for a user group.
///
/// # Examples
/// User groups are identified by their type and provider-assigned ID.
/// Currently, only LDAP groups are supported; these are represented as
/// `remote.group-name`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserGroupId(String);

impl UserGroupId {
    /// Creates a canonicalized user group ID.
    pub fn new<IS: Into<String>>(id: IS) -> Self {
        UserGroupId(id.into())
    }
    /// Gets the group ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
