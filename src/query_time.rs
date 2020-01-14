use crate::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents an absolute or relative time sent to an ExtraHop appliance
/// as part of a query.
///
/// # Usage
/// `QueryTime` instances should generally be constructed using the conversion
/// traits, rather than directly.
///
/// ```rust
/// # use extrahop::QueryTime;
/// let _time: QueryTime = (-30000 as i64).into();
/// let _other: QueryTime = "-30m".into();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTime(String);

impl QueryTime {
    /// Returns `true` if the query time is relative to the appliance's "now".
    ///
    /// # Examples
    /// ```rust
    /// # use extrahop::QueryTime;
    /// assert!(QueryTime::from("-30m").is_relative());
    /// assert!(QueryTime::from(0).is_relative());
    /// ```
    pub fn is_relative(&self) -> bool {
        self.0 == "0" || self.0.starts_with('-')
    }

    /// Returns `true` if the query time is measured from the Unix epoch.
    pub fn is_absolute(&self) -> bool {
        !self.is_relative()
    }
}

impl Default for QueryTime {
    fn default() -> Self {
        Self("0".to_string())
    }
}

impl From<i64> for QueryTime {
    fn from(val: i64) -> Self {
        Self(format!("{}", val))
    }
}

impl<'a> From<&'a str> for QueryTime {
    fn from(val: &str) -> Self {
        Self::from(String::from(val))
    }
}

/// Convert a string to a query time. This may convert the query time to a
/// number if doing so would not change readability in the serialized form.
impl From<String> for QueryTime {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl FromStr for QueryTime {
    type Err = crate::Error;

    fn from_str(v: &str) -> Result<Self> {
        Ok(Self(v.to_string()))
    }
}
