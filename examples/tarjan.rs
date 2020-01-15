use extrahop::activitymap::{Edge, Query, Response};
use extrahop::{ApiResponse, Client, Oid};
use petgraph::algo::tarjan_scc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Define the API client. No connection is made, as all requests go over HTTPS.
    // However, the client can be reused to make many requests.
    let client = Client::new("YOUR-HOST", "YOUR-KEY")?;

    let query = Query::builder()
        .from(-30000)
        .walks(vec![Default::default()])
        .build()
        .unwrap();

    let rsp = client
        .post("v1/activitymaps/query")?
        .json(&query)
        .send()
        .await?
        .validate_and_read::<Response>()
        .await?;

    if !rsp.is_complete() {
        println!("Warning; topology may be incomplete");
    }

    let graph = petgraph::Graph::<Oid, Edge>::from(rsp);

    let sccs = tarjan_scc(&graph);

    for scc in &sccs {
        println!("{:?}", scc);
    }

    let largest_component = sccs
        .iter()
        .max_by_key(|i| i.len())
        .map(|c| c.len())
        .unwrap_or(0);

    println!(
        "Reduced {} devices into {} components, largest was {} elements",
        graph.node_count(),
        sccs.len(),
        largest_component
    );

    Ok(())
}
