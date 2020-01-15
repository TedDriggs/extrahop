//! This example gets an activity map and then writes the edges out as tab-separated values.

#[cfg(feature = "topology")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::io;

    use extrahop::activitymap::{query, Edge, Query, Response, Source, Walk};
    use extrahop::{ApiResponse, Client};

    fn write_edge(f: &mut dyn io::Write, edge: &Edge) -> io::Result<()> {
        writeln!(
            f,
            "{from}\t{to}\t{weight}",
            from = edge.from.as_url_part(),
            to = edge.to.as_url_part(),
            weight = edge.weight
        )
    }

    let client = Client::new("your-extrahop", "YOUR-KEY")?;

    // Create topology query
    let request = Query::builder()
        .from("-1w")
        .walks(vec![Walk {
            origins: vec![Source::device_group(1)].into(),
            steps: vec![Default::default()],
            ..Default::default()
        }])
        .edge_annotations(vec![query::EdgeAnnotation::Protocols])
        .build()
        .unwrap();

    let response = client
        .post("v1/activitymaps/query")?
        .json(&request)
        .send()
        .await?
        .validate_and_read::<Response>()
        .await?;

    let mut stdout = io::stdout();
    for edge in &response {
        write_edge(&mut stdout, edge).unwrap();
    }

    Ok(())
}

#[cfg(not(feature = "topology"))]
fn main() {}
