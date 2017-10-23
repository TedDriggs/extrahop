//! Types for creating an activity map query request.

use std::ops::Index;

use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;

use ::Oid;
use activitymap::rsp::Appearance;

/// Envelope for an ad-hoc activity map query. If constructed with struct literal
/// syntax, `Request::default()` _must_ be used to ensure source compatibility with
/// future library updates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Builder)]
#[builder(default)]
pub struct Request {
    /// The absolute or relative timestamp at which the query should start.
    pub from: i64,

    /// The absolute or relative timestmap at which the query should end. If not set,
    /// defaults to the current packet time of the appliance.
    pub until: i64,

    /// The traversals that should be performed across the topology. Results from all
    /// walks will be merged into a single set of edges in the response. 
    pub walks: Vec<Walk>,

    /// The set of metrics should drive the weight of an edge.
    pub weight_strategy: WeightStrategy,

    /// The additional data to return for each edge.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edge_annotations: Vec<EdgeAnnotation>,
}

/// Find a step configuration for an `rsp::Appearance` from the edge list in
/// a query response.
impl Index<Appearance> for Request {
    type Output = Step;

    fn index(&self, idx: Appearance) -> &Self::Output {
        &self.walks[idx.walk as usize].steps[idx.step as usize]
    }
}

impl From<Walk> for Request {
    fn from(walk: Walk) -> Self {
        Request {
            walks: vec![walk],
            ..Default::default()
        }
    }
}

/// The type of metrics that should be used to compute edge weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightStrategy {
    Bytes,
    Connections,
    Turns,
}

impl Default for WeightStrategy {
    fn default() -> Self {
        WeightStrategy::Bytes
    }
}

/// Flags to opt into additional data about the topology from the appliance. 
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeAnnotation {
    /// Causes the response to include an array for each edge which lists each
    /// walk and step index which traversed the selected edge.
    Appearances,

    /// Causes the response to include an array for each edge breaking down the
    /// total edge weight by protocol.
    Protocols,
}

/// A set of steps from one or more starting points which build
/// a directed graph topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
#[builder(setter(into))]
pub struct Walk {
    /// The starting device(s) for the walk.
    pub origins: WalkOrigin,

    /// The ordered set of steps to take away from the walk origins.
    pub steps: Vec<Step>,
}

/// Sets the origins for a walk.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum WalkOrigin {
    /// Starts the walk at every compatible device.
    All,
    /// Starts the walk from the specified devices or compatible members of the
    /// specified groups.
    Specific(Vec<Source>),
}

impl Serialize for WalkOrigin {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match *self {
            WalkOrigin::All => {
                #[derive(Serialize)]
                struct AllDevices {
                    object_type: &'static str
                };

                (AllDevices {
                    object_type: "all_devices"
                }).serialize(s)
            },
            WalkOrigin::Specific(ref sources) => {
                let mut seq = s.serialize_seq(Some(sources.len()))?;
                for source in sources {
                    seq.serialize_element(source)?;
                }

                seq.end()
            }
        }
    }
}

impl From<Vec<Source>> for WalkOrigin {
    fn from(val: Vec<Source>) -> Self {
        WalkOrigin::Specific(val)
    }
}

/// Represents a metric source, such as a device or device group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub object_type: ObjectType,
    pub object_id: Oid,
}

impl Source {
    /// Create a new `Source` instance.
    pub fn new(object_type: ObjectType, id: Oid) -> Self {
        Source {
            object_type: object_type,
            object_id: id,
        }
    }

    pub fn device(id: Oid) -> Self {
        Source::new(ObjectType::Device, id)
    }

    pub fn device_group(id: Oid) -> Self {
        Source::new(ObjectType::DeviceGroup, id)
    }
}

/// Type of a metric source object which is compatible with the topology API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectType {
    /// An individual endpoint.
    Device,
    /// A user-defined set of devices.
    DeviceGroup,
    /// A system-defined set of all devices speaking a given protocol
    /// during the queried window.
    ActivityGroup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
#[serde(default)]
#[builder(default, setter(into))]
pub struct Step {
    /// If non-empty, limits the protocol and peer role of edges found
    /// during this step.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<Relationship>,

    /// If non-empty, limits the edges found during this step to those whose
    /// devices are in the specified groups.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub peer_in: Vec<Source>,

    /// If non-empty, limits the edges found during this step to those whose
    /// devices are not in the specified groups. If both this property and
    /// `member_of` are present in a step, then an edge must satisfy both
    /// checks to be included in the response.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub peer_not_in: Vec<Source>,
}

impl Default for Step {
    fn default() -> Self {
        Step {
            relationships: vec![],
            peer_in: vec![],
            peer_not_in: vec![],
        }
    }
}

/// A combination of protocol and peer role which can match a connection between devices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Relationship {
    /// The role of the peer device in the relationship to be discovered.
    #[serde(default, skip_serializing_if = "Role::is_default")]
    pub role: Role,
    /// The protocol that must be spoken between the start device and peer to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Protocol>,
}

impl Relationship {
    pub fn new<P: Into<Protocol>>(protocol: P, role: Role) -> Self {
        Self {
            role: role,
            protocol: Some(protocol.into()),
        }
    }
}

impl From<Protocol> for Relationship {
    fn from(val: Protocol) -> Self {
        Relationship::new(val, Role::Any)
    }
}

/// Create a new relationship matching any protocol with the specified peer role.
impl From<Role> for Relationship {
    fn from(role: Role) -> Self {
        Relationship {
            role: role,
            protocol: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Client,
    Server,
    Any,
}

impl Role {
    fn is_default(&self) -> bool {
        *self == Role::Any
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::Any
    }
}

/// A protocol name that will be used to filter the edges traversed during the walk.
/// Note that unlike `rsp::ProtocolStack`, this is a single string and not a full stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protocol(pub String);

impl<'a> From<&'a str> for Protocol {
    fn from(val: &str) -> Self {
        Protocol(String::from(val))
    }
}

impl From<String> for Protocol {
    fn from(val: String) -> Self {
        Protocol(val)
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use super::WalkOrigin;

    #[test]
    fn source_list_serialize_all_devices() {
        assert_eq!(
            r#"{"object_type":"all_devices"}"#,
            serde_json::to_string(&WalkOrigin::All).unwrap()
        );
    }
}