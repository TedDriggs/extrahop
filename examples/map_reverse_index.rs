//! This example loads an activity map topology, then creates a map from each observed OID to
//! the edges where that device appears.

#[cfg(feature = "topology")]
use std::collections::HashMap;

#[cfg(feature = "topology")]
use extrahop::activitymap::{Edge, Query, Response};
#[cfg(feature = "topology")]
use extrahop::{ApiResponse, Client, Oid};

/// An activity map body with a map of node IDs to the edges in which they appear.
#[derive(Clone)]
#[cfg(feature = "topology")]
struct IndexedTopology {
    topology: Response,
    node_map: HashMap<Oid, Vec<usize>>,
}

#[cfg(feature = "topology")]
impl IndexedTopology {
    /// Gets read access to the topology returned by the appliance.
    pub fn topology(&self) -> &Response {
        &self.topology
    }

    /// Get the node IDs in the topology.
    pub fn nodes(&self) -> Vec<&Oid> {
        self.node_map.keys().collect()
    }

    /// Gets the edges which contain the specified object ID.
    pub fn get(&self, id: Oid) -> Vec<&Edge> {
        self.node_map
            .get(&id)
            .map(|indices| {
                indices
                    .iter()
                    .map(|index| self.topology.edges.get(*index).unwrap())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(feature = "topology")]
impl From<Response> for IndexedTopology {
    fn from(map: Response) -> Self {
        let mut node_map = HashMap::new();

        // Walk the edges, populating the node map as we go
        {
            let mut add_to_index = |index, id| {
                let entry = node_map.entry(id).or_insert_with(Vec::new);
                entry.push(index);
            };

            for (index, edge) in map.iter().enumerate() {
                add_to_index(index, edge.from.clone());
                add_to_index(index, edge.to.clone());
            }
        }

        Self {
            topology: map,
            node_map,
        }
    }
}

#[cfg(feature = "topology")]
impl<'de> serde::Deserialize<'de> for IndexedTopology {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(IndexedTopology::from(Response::deserialize(deserializer)?))
    }
}

#[cfg(feature = "topology")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Define the API client. No connection is made, as all requests go over HTTPS.
    // However, the client can be reused to make many requests.
    let client = Client::new_appliance("your-host", "YOUR-KEY".into(), Default::default()).await?;

    let query = Query::builder().from(-30000).build()?;

    let rsp: IndexedTopology = client
        .post("v1/activitymaps/query")?
        .json(&query)
        .send()
        .await?
        .validate_and_read::<IndexedTopology>()
        .await?;

    if !rsp.topology().is_complete() {
        println!("Warning; topology may be incomplete");
    }

    let nodes = rsp.nodes();
    for node in nodes.into_iter().take(1) {
        for edge in rsp.get(node.clone()) {
            println!("{:?}", edge);
        }
    }

    Ok(())
}

#[cfg(not(feature = "topology"))]
fn main() {}
