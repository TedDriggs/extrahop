use regex::Regex;

use ErrorKind;

lazy_static! {
    static ref REL_VALIDATOR: Regex = Regex::new(r"^-\d+(?:ms|s|m|h|d|w|y)?$")
        .expect("Static regex should be valid");
}

/// Represents an absolute or relative time sent to an ExtraHop appliance
/// as part of a query.
///
/// # Usage
/// `QueryTime` instances should generally be constructed using the conversion
/// traits, rather than directly.
///
/// ```rust
/// # use extrahop::QueryTime;
/// let _time: QueryTime = (-30000).into();
/// let _other: QueryTime = "-30m".into();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryTime {
    /// A number of milliseconds. Positive values are assessed since the Unix
    /// epoch, while non-positive values (including 0) are assessed relative to
    /// the appliance's "now" value.
    Milliseconds(i64),

    /// A possibly-unitized amount of time, expressed as a string.
    Unitized(String),
}

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
        // Non-positive values are relative times.
        match *self {
            QueryTime::Milliseconds(ref val) => *val <= 0,
            QueryTime::Unitized(ref val) => val.starts_with("-") || val == "0"
        }
    }

    /// Returns `true` if the query time is measured from the Unix epoch.
    pub fn is_absolute(&self) -> bool {
        !self.is_relative()
    }

    /// Check that a query time specification will be accepted by the appliance,
    /// returning the unmodified query time if so, and otherwise returning an error.
    ///
    /// This function internally calls `QueryTime::is_valid`.
    pub fn validate(self) -> ::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            bail!(ErrorKind::QueryTimeParseError)
        }
    }

    /// Check if a query time is valid.
    ///
    /// # Caution
    /// This method attempts to reimplement the appliance's own format checking
    /// for unitized time strings, so it is possible that future firmware will
    /// accept values not accepted by this method.
    ///
    /// # Validation
    /// * Numeric time values are allowed without further inspection
    /// * Relative unitized time values must adhere to this expression: `^-\d+(?:ms|s|m|h|d|y)?$`
    /// * Absolute unitized time values must be composed of only digits
    ///
    /// # Examples
    /// ```rust
    /// # use extrahop::QueryTime;
    /// // Valid
    /// assert!(QueryTime::from(0).is_valid());
    /// assert!(QueryTime::from(-25000).is_valid());
    /// assert!(QueryTime::from("-1m").is_valid());
    /// assert!(QueryTime::from("-50").is_valid());
    /// assert!(QueryTime::from("-10ms").is_valid());
    /// assert!(QueryTime::from("0").is_valid());
    ///
    /// // Invalid
    /// assert!(!QueryTime::from("10m").is_valid());
    /// assert!(!QueryTime::from("1-0").is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        match *self {
            QueryTime::Milliseconds(_) => true,
            QueryTime::Unitized(ref val) => {
                // Check if the time is a relative string, because that's the 95% case
                // If that fails, then look to see if we've received an absolute time
                REL_VALIDATOR.is_match(&val) || val.chars().all(|c| c.is_digit(10))
            }
        }
    }
}

impl Default for QueryTime {
    fn default() -> Self {
        QueryTime::Milliseconds(0)
    }
}

impl From<i64> for QueryTime {
    fn from(val: i64) -> Self {
        QueryTime::Milliseconds(val)
    }
}

impl<'a> From<&'a str> for QueryTime {
    fn from(val: &str) -> Self {
        QueryTime::from(String::from(val))
    }
}

/// Convert a string to a query time. This may convert the query time to a
/// number if doing so would not change readability in the serialized form.
impl From<String> for QueryTime {
    fn from(val: String) -> Self {
        if val.chars().all(|c| c.is_digit(10) || c == '-') {
            if let Ok(ms) = val.parse() {
                QueryTime::Milliseconds(ms)
            } else {
                QueryTime::Unitized(val)
            }
        } else {
            QueryTime::Unitized(val)
        }
    }
}