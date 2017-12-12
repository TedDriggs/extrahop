extern crate extrahop;
extern crate petgraph;

use extrahop::{ApiKey, ApiResponse, Client, Oid};
use extrahop::activitymap::{Edge, Query, Response};
use petgraph::algo::tarjan_scc;

fn main() {
    // Define the API client. No connection is made, as all requests go over HTTPS.
    // However, the client can be reused to make many requests.
    let client = Client::new("YOUR-HOST", ApiKey::new("YOUR-KEY"));

    let query = Query {
        from: (-30000).into(),
        walks: vec![Default::default()],
        ..Query::default()
    };

    let rsp: Response = client
        .post("activitymaps/query")
        .json(&query)
        .send()
        .validate_and_read()
        .expect("Query should produce a valid response");

    if !rsp.is_complete() {
        println!("Warning; topology may be incomplete");
    }

    let graph = petgraph::Graph::<Oid, Edge>::from(rsp);

    let sccs = tarjan_scc(&graph);

    for scc in &sccs {
        println!("{:?}", scc);
    }

    let largest_component = sccs.iter()
        .max_by_key(|i| i.len())
        .map(|c| c.len())
        .unwrap_or(0);

    println!(
        "Reduced {} devices into {} components, largest was {} elements",
        graph.node_count(),
        sccs.len(),
        largest_component
    );
}