//! Types for working with activity map topologies.
//!
//! This module contains the request and response types needed to interact with
//! `/api/v1/activitymaps/query`.
//!
//! # Usage
//! Queries can be constructed using struct literals or with builders from
//! the `req` module.
//!
//! ## Using Builders
//! ```rust
//! use extrahop::Oid;
//! use extrahop::activitymap::req::*;
//!
//! // Create a request for the last half hour, starting from device 15 and
//! // finding all its immediate peers. The default weight strategy will be
//! // used, and no extra annotations were requested.
//! let _ = RequestBuilder::default()
//!             .from(-30000)
//!             .walks(vec![
//!                 WalkBuilder::default()
//!                     .origins(vec![Source::device(Oid::new(15))])
//!                     .steps(vec![Step::default()])
//!                     .build()
//!                     .unwrap()
//!             ])
//!             .build()
//!             .unwrap();
//! ```

pub mod req;
pub mod rsp;

pub use self::req::{Source, Step, Walk, WalkOrigin};

#[doc(inline)]
pub use self::req::Request;

#[doc(inline)]
pub use self::rsp::Response;

#[cfg(test)]
mod tests {
    use serde_json;

    use ::Oid;
    use super::req::{self, Step, Relationship, Role, Source};

    #[test]
    fn it_works() {
        let request = req::Request {
            from: 0.into(),
            walks: vec![req::Walk {
                origins: vec![Source::device(Oid::new(14))].into(),
                steps: vec![
                    Step {
                        relationships: vec![Relationship::new("HTTP", Role::Server)],
                        ..Default::default()
                    },
                ]
            }],
            edge_annotations: vec![req::EdgeAnnotation::Protocols],
            ..Default::default()
        };

        println!("{}", serde_json::to_string_pretty(&request).unwrap());
    }
}
