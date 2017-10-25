//! Types for creating and serializing an activity map query request.
//!
//! # Serialization & Deserialization
//! All types in this module support serialization and deserialization via `serde`. 
//! Types generally try to only serialize properties that differ from the backend 
//! defaults; this should reduce the size of the serialized object and improve 
//! readability.

use std::ops::Index;

use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;

use ::{Builder, Oid, QueryTime};
use activitymap::rsp::Appearance;

/// Envelope for an ad-hoc activity map query.
///
/// # Construction
/// If constructed with struct literal syntax, `Query::default()` _must_ 
/// be used to ensure source compatibility with future library updates.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
#[builder(default, setter(into))]
#[serde(default)]
pub struct Query {
    /// The absolute or relative timestamp at which the query should start.
    pub from: QueryTime,

    /// The absolute or relative timestmap at which the query should end. If not set,
    /// defaults to the current packet time of the appliance.
    pub until: QueryTime,

    /// The traversals that should be performed across the topology. Results from all
    /// walks will be merged into a single set of edges in the response. 
    pub walks: Vec<Walk>,

    /// The set of metrics should drive the weight of an edge.
    pub weighting: Weighting,

    /// The additional data to return for each edge.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edge_annotations: Vec<EdgeAnnotation>,
}

/// Find a step configuration for an `rsp::Appearance` from the edge list in
/// a query response.
impl Index<Appearance> for Query {
    type Output = Step;

    fn index(&self, idx: Appearance) -> &Self::Output {
        &self.walks[idx.walk as usize].steps[idx.step as usize]
    }
}

impl From<Walk> for Query {
    fn from(walk: Walk) -> Self {
        Query {
            walks: vec![walk],
            ..Default::default()
        }
    }
}

impl Builder for Query {
    type Builder = QueryBuilder;
}

/// The type of metrics that should be used to compute edge weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weighting {
    /// The number of bytes transferred in both directions between the two peers.
    /// This is the default strategy.
    Bytes,
    
    /// The number of connections *established* during the time interval.
    ///
    /// This does not include connections opened before the query interval,
    /// so results may be lower than expected, especially for protocols with
    /// long-lived connections.
    Connections,
    Turns,
}

impl Default for Weighting {
    fn default() -> Self {
        Weighting::Bytes
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

impl Default for Walk {
    fn default() -> Self {
        Walk {
            origins: WalkOrigin::All,
            steps: vec![],
        }
    }
}

impl Builder for Walk {
    type Builder = WalkBuilder;
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
                /// The backend API overloads the "object_type" field to accept
                /// "all_devices" in walk origins. Since "all_devices" isn't a
                /// valid type in other contexts, we hide that knowledge here.
                #[derive(Serialize)]
                struct AllDevices {
                    object_type: &'static str
                };

                (vec![AllDevices {
                    object_type: "all_devices"
                }]).serialize(s)
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
    pub fn new<I: Into<Oid>>(object_type: ObjectType, id: I) -> Self {
        Source {
            object_type: object_type,
            object_id: id.into(),
        }
    }

    /// Create a new `Source` instance for a device.
    pub fn device<I: Into<Oid>>(id: I) -> Self {
        Source::new(ObjectType::Device, id.into())
    }

    /// Create a new `Source` instance for a device group.
    pub fn device_group<I: Into<Oid>>(id: I) -> Self {
        Source::new(ObjectType::DeviceGroup, id.into())
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

/// A traversal instruction which can find new edges or protocols to include in
/// an activity map.
///
/// Each step moves from all the devices found in the previous step along the 
/// specified relationships, and then prunes the found edges based on additional
/// filters such as `peer_in` and `peer_not_in`.
///
/// # Notes
/// * If `relationships` is set to a single protocol and role pair, such as "http server",
///   it is not necessary to also apply a `peer_in` filter for the HTTP Servers activity
///   group.
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

impl Builder for Step {
    type Builder = StepBuilder;
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
    /// Create a new `Relationship` for the specified protocol and role.
    pub fn new<P: Into<Protocol>>(protocol: P, role: Role) -> Self {
        Self {
            role: role,
            protocol: Some(protocol.into()),
        }
    }
}

/// Create a relationship matching all peers over the specified protocol, regardless
/// of which device fulfilled which role.
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

/// The role an endpoint is able to fill in a network transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
///
/// Unlike `rsp::ProtocolStack`, this is a single string and not a full stack.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Protocol(String);

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
            r#"[{"object_type":"all_devices"}]"#,
            serde_json::to_string(&WalkOrigin::All).unwrap()
        );
    }
}