/// Represents an absolute or relative time sent to an ExtraHop appliance
/// as part of a query.
///
/// # Usage
/// `QueryTime` instances should generally be constructed using the conversion
/// traits, rather than directly.
///
/// ```rust
/// let _time: QueryTime = -30000.into();
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

    /// Check that a query time specification will be accepted by the appliance.
    ///
    /// # Caution
    /// This method attempts to reimplement the appliance's own format checking
    /// for unitized time strings, so it is possible that future firmware will
    /// accept values not accepted by this method.
    ///
    /// # Validation
    /// * Numeric time values are allowed without further inspection
    /// * Unitized time values must adhere to this expression: `^-?\d+(?:ms|s|m|h|d|y)?$`
    pub fn validate(self) -> ::Result<Self> {
        Ok(self)
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
        QueryTime::Unitized(String::from(val))
    }
}

impl From<String> for QueryTime {
    fn from(val: String) -> Self {
        QueryTime::Unitized(val)
    }
}