//! This example loads an activity map topology, then creates a map from each observed OID to 
//! the edges where that device appears.

extern crate extrahop;
extern crate reqwest;

extern crate serde;

use std::collections::HashMap;

use extrahop::{ApiKey, ApiResponse, Client, Oid};
use extrahop::activitymap::{Query, Response, Edge};

/// An activity map body with a map of node IDs to the edges in which they appear.
#[derive(Clone)]
struct IndexedTopology {
    topology: Response,
    node_map: HashMap<Oid, Vec<usize>>,
}

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
        self.node_map.get(&id).map(|indices| {
            indices
                .iter()
                .map(|index| self.topology.edges.get(*index).unwrap())
                .collect()
        }).unwrap_or_default()
    }
}

impl From<Response> for IndexedTopology {
    fn from(map: Response) -> Self {
        let mut node_map = HashMap::new();

        // Walk the edges, populating the node map as we go
        {
            let mut add_to_index = |index, id| {
                let entry = node_map.entry(id).or_insert(Vec::new());
                entry.push(index);
            };

            for (index, edge) in map.iter().enumerate() {
                add_to_index(index, edge.from.clone());
                add_to_index(index, edge.to.clone());
            }
        }

        Self {
            topology: map,
            node_map: node_map,
        }
    }
}

impl<'de> serde::Deserialize<'de> for IndexedTopology {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(IndexedTopology::from(Response::deserialize(deserializer)?))
    }
}

fn main() {
    // Define the API client. No connection is made, as all requests go over HTTPS.
    // However, the client can be reused to make many requests.
    let client = Client::new("your-host", ApiKey::new("YOUR-KEY"));

    let query = Query {
        from: (-30000).into(),
        ..Default::default()
    };

    let rsp: IndexedTopology = client
        .post("/activitymaps/query")
        .json(&query)
        .send()
        .validate_and_read()
        .expect("Query should produce a valid response");

    if !rsp.topology().is_complete() {
        println!("Warning; topology may be incomplete");
    }

    let nodes = rsp.nodes();
    for node in nodes.into_iter().take(1) {
        for edge in rsp.get(node.clone()) {
            println!("{:?}", edge);
        }
    }
}