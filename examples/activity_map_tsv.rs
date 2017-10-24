//! This example gets an activity map and then writes the edges out as tab-separated values.

extern crate extrahop;
extern crate reqwest;

use std::io;

use extrahop::{ApiKey, ApiResponse, Client, Result};
use extrahop::activitymap::{self, req, Edge, Response};

fn write_edge(f: &mut io::Write, edge: &Edge) -> io::Result<()> {
    writeln!(
        f,
        "{from}\t{to}\t{weight}",
        from = edge.from.as_url_part(),
        to = edge.to.as_url_part(),
        weight = edge.weight
    )
}

fn main() {
    let client = Client::new("your-extrahop", ApiKey::new("YOUR-KEY"));

    // Create topology query
    let request = req::RequestBuilder::default()
        .from("-1w")
        .walks(vec![
            req::Walk {
                origins: vec![req::Source::device_group(1)].into(),
                steps: vec![Default::default()],
                ..Default::default()
            },
        ])
        .edge_annotations(vec![req::EdgeAnnotation::Protocols])
        .build()
        .unwrap();

    let response: Result<Response> = client
        .post("/activitymaps/query")
        .json(&request)
        .send()
        .validate_and_read();

    if let Ok(edges) = response {
        let mut stdout = io::stdout();
        for edge in &edges {
            write_edge(&mut stdout, edge);
        }
    }
}