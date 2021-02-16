use serde::{Serialize, Serializer};
use std::{fmt, num::NonZeroU64};

#[derive(Debug, Clone)]
enum Inner {
    Timestamp(NonZeroU64),
    Now,
    MsAgo(NonZeroU64),
    RelativeUnits(String),
}

impl Serialize for Inner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Inner::Now => 0.serialize(serializer),
            Inner::Timestamp(ts) => ts.serialize(serializer),
            Inner::MsAgo(ms) => ((ms.get() as i64) * -1).serialize(serializer),
            Inner::RelativeUnits(string) => string.serialize(serializer),
        }
    }
}

/// Represents an absolute or relative time sent to an ExtraHop API
/// as part of a query.
///
/// # Default
/// By default, `QueryTime` refers to the current time as known to the called API;
/// this is represented in API requests as `0`.
///
/// # Usage
/// `QueryTime` instances should generally be constructed using the conversion
/// traits, rather than directly.
///
/// ```rust
/// # use extrahop::QueryTime;
/// let _time: QueryTime = (-30000i64).into();
/// let _other: QueryTime = "-30m".into();
/// ```
#[derive(Clone, Serialize)]
pub struct QueryTime(Inner);

impl fmt::Debug for QueryTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl QueryTime {
    /// Returns `true` if the query time is relative to the appliance's "now".
    ///
    /// # Examples
    /// ```rust
    /// # use extrahop::QueryTime;
    /// assert!(QueryTime::from("-30m").is_relative());
    /// assert!(QueryTime::from(0u64).is_relative());
    /// ```
    pub fn is_relative(&self) -> bool {
        match self.0 {
            Inner::Timestamp(_) => false,
            Inner::Now | Inner::MsAgo(_) | Inner::RelativeUnits(_) => true,
        }
    }

    /// Returns `true` if the query time is measured from the Unix epoch.
    pub fn is_absolute(&self) -> bool {
        !self.is_relative()
    }
}

impl Default for QueryTime {
    fn default() -> Self {
        Self(Inner::Now)
    }
}

impl From<u64> for QueryTime {
    fn from(val: u64) -> Self {
        if let Some(ts) = NonZeroU64::new(val) {
            Self(Inner::Timestamp(ts))
        } else {
            Self(Inner::Now)
        }
    }
}

impl From<i64> for QueryTime {
    fn from(val: i64) -> Self {
        match val {
            0 => Self(Inner::Now),
            x if x > 0 => Self(Inner::Timestamp(NonZeroU64::new(x as u64).unwrap())),
            x if x < 0 => Self(Inner::MsAgo(NonZeroU64::new(x.abs() as u64).unwrap())),
            _ => unreachable!(),
        }
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
        Self(Inner::RelativeUnits(val))
    }
}

#[cfg(test)]
mod tests {
    use super::QueryTime;
    use serde_json;

    #[test]
    fn serialize_time_string() {
        assert_eq!(
            r#""-30m""#,
            serde_json::to_string(&QueryTime::from("-30m")).unwrap(),
        );
    }

    #[test]
    fn serialize_default() {
        assert_eq!("0", serde_json::to_string(&QueryTime::default()).unwrap());
    }

    #[test]
    fn serialize_timestamp() {
        assert_eq!(
            "123",
            serde_json::to_string(&QueryTime::from(123u64)).unwrap(),
        );
    }

    #[test]
    fn serialize_ms_ago() {
        assert_eq!(
            "-123",
            serde_json::to_string(&QueryTime::from(-123i64)).unwrap()
        )
    }
}
