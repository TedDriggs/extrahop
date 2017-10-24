//! Types for deserializing a response from `/api/v1/activitymaps/query`

use std::{cmp, fmt, vec, slice};
use std::collections::HashSet;

use serde_json;

use Oid;

/// A successful response to a single topology API request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Response {
    /// Non-fatal errors encountered during the construction of the map.
    /// Items in this list indicate that the returned topology may be incomplete.
    pub warnings: Vec<Error>,
    /// The absolute packet time at which data starts for the response.
    pub from: u64,
    /// The absolute packet time at which data ends for the response.
    pub until: u64,
    /// The collection of edges which matched the activity map query.
    pub edges: Vec<Edge>,
}

impl Response {
    /// Computes the set of nodes in the response.
    pub fn nodes(&self) -> HashSet<Oid> {
        let mut oids = HashSet::new();
        for edge in &self.edges {
            oids.insert(edge.from.clone());
            oids.insert(edge.to.clone());
        }

        oids
    }

    /// Gets a slice iterator over the edges
    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Edge> {
        self.edges.iter()
    }

    /// Checks that there are no warnings which indicate an incomplete response
    /// from the appliance.
    pub fn is_complete(&self) -> bool {
        self.warnings.is_empty()
    }
}

impl Default for Response {
    fn default() -> Self {
        Response {
            warnings: vec![],
            from: 0,
            until: 0,
            edges: vec![],
        }
    }
}

impl IntoIterator for Response {
    type Item = Edge;
    type IntoIter = vec::IntoIter<Edge>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges.into_iter()
    }
}

impl<'a> IntoIterator for &'a Response {
    type Item = &'a Edge;
    type IntoIter = slice::Iter<'a, Edge>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An error or warning returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    /// A human-friendly message describing the error that occurred.
    pub message: String,

    /// A machine-friendly string that identifies the type of error.
    #[serde(rename = "type")]
    pub error_type: String,

    /// A bag of properties that can be used in conjunction with the error type
    /// to construct a richer error message.
    #[serde(default)]
    pub(crate) properties: Option<serde_json::Value>,
}

impl Error {
    /// Create a new error with no properties.
    pub fn new<M: Into<String>, E: Into<String>>(message: M, error_type: E) -> Self {
        Error {
            message: message.into(),
            error_type: error_type.into(),
            properties: None,
        }
    }
}

/// A connection between two nodes in a directed graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    /// The "client" device in the edge. This may be the device on which
    /// the step *ended* if a server-to-client relationship was specified.
    pub from: Oid,

    /// The "server" device in the edge. This may be the device on which
    /// the step *started* if a server-to-client relationship was specified.
    pub to: Oid,

    /// The "importance" of the edge; larger numbers are more important.
    pub weight: usize,
    
    /// Additional data about the edge which was asked for in the request.
    #[serde(default)]
    pub annotations: EdgeAnnotations,
}

/// Additional data about the edge which can be asked for in the request.
/// Properties should have a value of `Some` when their key was present
/// in the request, though the contents may themselves be empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EdgeAnnotations {
    /// The list of walk/step pairs which traversed this edge during map
    /// construction.
    pub appearances: Option<Vec<Appearance>>,

    /// The per-protocol contributions to the edge's weight.
    pub protocols: Option<Vec<ProtocolAnnotation>>,
}

/// A walk index and step index into the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct Appearance {
    /// The index of the walk that contributed to this appearance.
    pub walk: u16,
    /// The index of the step in the walk that contributed to this appearance.
    pub step: u16,
}

impl Appearance {
    /// Create a new `Appearance` for a specific walk and step.
    pub fn new(walk: u16, step: u16) -> Self {
        Appearance {
            walk: walk,
            step: step,
        }
    }
}

impl PartialOrd for Appearance {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let walk = self.walk.cmp(&other.walk);
        if walk == cmp::Ordering::Equal {
            Some(self.step.cmp(&other.step))
        } else {
            Some(walk)
        }
    }
}

/// An annotation connecting a protocol to the weight that it added to an
/// edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolAnnotation {
    /// The amount of weight on the protocol. The units depend on the
    /// request configuration.
    pub weight: u32,
    /// The protocol which contributed the specified weight.
    pub protocol: ProtocolStack,
}

/// The stack of protocols for this part of the edge.
///
/// This will look as follows:
///
/// ```text
/// [L3Protocol, L4Protocol, L7Protocol+]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolStack(Vec<String>);

impl fmt::Display for ProtocolStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = &self.0;
        let len = val.len();

        if val.is_empty() {
            write!(f, "OTHER")
        } else if len > 1 && val[len - 1] == "OTHER" {
            write!(f, "{}", val[len - 2])
        } else {
            write!(f, "{}", val[len - 1])
        }
    }
}

/// Convenience impl for testing `ProtocolStack`
impl From<Vec<&'static str>> for ProtocolStack {
    fn from(vals: Vec<&'static str>) -> Self {
        ProtocolStack(vals.into_iter().map(String::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{Appearance, ProtocolStack};

    #[test]
    fn protocol_fmt_http() {
        assert_eq!(
            "HTTP",
            &format!("{}", ProtocolStack::from(vec!["IPv4", "TCP", "HTTP"]))
        );
        assert_eq!(
            "HTTP",
            &format!("{}", ProtocolStack::from(vec!["IPv6", "TCP", "HTTP"]))
        );
    }

    #[test]
    fn protocol_fmt_unknowns() {
        assert_eq!(
            "TCP",
            &format!("{}", ProtocolStack::from(vec!["IPv4", "TCP", "OTHER"]))
        );
        assert_eq!("OTHER", &format!("{}", ProtocolStack::from(vec!["OTHER"])));
    }

    /// Appearances are sorted by walk, then by step.
    #[test]
    fn order_appearances() {
        let mut items = vec![
            Appearance::new(1, 2),
            Appearance::new(0, 2),
            Appearance::new(1, 0),
            Appearance::new(0, 1),
        ];

        items.sort();

        assert_eq!(items, vec![
            Appearance::new(0, 1),
            Appearance::new(0, 2),
            Appearance::new(1, 0),
            Appearance::new(1, 2),
        ]);
    }
}