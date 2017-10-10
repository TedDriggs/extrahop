//! Types for interacting with dashboard sharing state.
//!
//! Dashboard sharing is controlled via `api/v1/dashboards/{id}/sharing`.

use std::collections::HashMap;
use std::default::Default;

use {Patch, Username, UserGroupId};

/// A set of permissions grantable to a user or group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Grants read access to a resource.
    #[serde(rename = "viewer")]
    Viewer,

    /// Grants read, edit, and sharing access to a resource.
    /// The ability to edit may be limited by the user's system role.
    #[serde(rename = "editor")]
    Editor,
}

fromstr_deserialize!(Role);

/// A representation of a sharing structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sharing<R> {
    /// The access level guaranteed to all authenticated users on the appliance.
    #[serde(skip_serializing_if="Option::is_none")]
    pub anyone: Option<R>,

    /// A map of users to granted access levels.
    #[serde(default = "HashMap::default", skip_serializing_if="HashMap::is_empty")]
    pub users: HashMap<Username, R>,

    /// A map of user groups to granted access levels.
    #[serde(default = "HashMap::default", skip_serializing_if="HashMap::is_empty")]
    pub groups: HashMap<UserGroupId, R>,
}

/// A snapshot of a resource's sharing state. This will be returned by `GET`
/// requests and can overwrite existing sharing state with a `PUT` request.
pub type SharingState = Sharing<Role>;

/// A delta update to a resource's sharing state.
///
/// A no-op patch can be created using `SharingPatch::default()`.
///
/// # Examples
/// Setting `anyone` to `None` will avoid updating the base access level for all
/// authenticated users. To unshare a dashboard from the "all users" group, set
/// the property to `Some(None)`.
///
/// ```rust
/// use extrahop::sharing::{Role, SharingPatch};
///
/// let make_private = SharingPatch {
///     anyone: Some(None),
///     ..SharingPatch::default()
/// };
/// ```
///
/// Sharing a dashboard to everyone is done using `Some(Some(Role::Viewer))`.
///
/// ```rust
/// # use extrahop::sharing::{Role, SharingPatch};
/// let make_public = SharingPatch {
///     anyone: Some(Some(Role::Viewer)),
///     ..SharingPatch::default()
/// };
/// ```
pub type SharingPatch = Sharing<Option<Role>>;

/// A no-op sharing update.
impl Default for Sharing<Option<Role>> {
    fn default() -> Self {
        Sharing {
            anyone: None,
            users: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}

impl Patch for Sharing<Option<Role>> {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::{self, FromIterator};

    use serde_json;

    use super::{SharingPatch, SharingState, Role};

    use {Username, UserGroupId};

    static SAMPLE_1: &'static str = r#"{
            "anyone": "viewer",
            "users": {
                "abirmingham": "editor",
                "botha": "editor",
                "dave": "editor",
                "ehd": "editor",
                "green": "editor",
                "mikelly": "editor",
                "mikem": "editor"
            },
            "groups": {}
        }"#;

    #[test]
    fn deserialize_sharing() {

        let parsed: SharingState = serde_json::from_str(SAMPLE_1).unwrap();

        assert_eq!(Some(&Role::Editor),
                   parsed.users.get(&Username::new("ehd".to_string())));
    }

    #[test]
    fn serialize_full_patch() {
        let change = SharingPatch {
            anyone: Some(None),
            users: HashMap::from_iter(vec![(Username::new("ehd"), None),
                                           (Username::new("mikelly"), Some(Role::Viewer))]),
            groups: HashMap::from_iter(iter::once((UserGroupId::new("remote.pm-team"),
                                                   Some(Role::Editor)))),
        };

        println!("{}", serde_json::to_string_pretty(&change).unwrap());
    }

    #[test]
    fn serialize_targeted() {
        let change = SharingPatch {
            users: HashMap::from_iter(iter::once((Username::new("ehd"), Some(Role::Editor)))),
            ..SharingPatch::default()
        };

        println!("{}", serde_json::to_string_pretty(&change).unwrap());
    }

    #[test]
    fn noop_patch() {
        // A no-op patch is an empty JSON object.
        assert_eq!("{}",
                   &serde_json::to_string(&SharingPatch::default()).unwrap());

        // And conversely, an empty JSON object is a no-op patch.
        assert_eq!(SharingPatch::default(),
                   serde_json::from_str::<SharingPatch>("{}").unwrap());
    }
}