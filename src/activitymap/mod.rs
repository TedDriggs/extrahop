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
//! use extrahop::activitymap::{self, Walk, Source, Step};
//!
//! // Create a request for the last half hour, starting from device 15 and
//! // finding all its immediate peers. The default weight strategy will be
//! // used, and no extra annotations were requested.
//! let _ = activitymap::Query::builder()
//!             .from(-30000)
//!             .walks(vec![
//!                 Walk::builder()
//!                     .origins(vec![Source::device(Oid::new(15))])
//!                     .steps(vec![Step::default()])
//!                     .build().unwrap()
//!             ])
//!             .build().unwrap();
//! ```

pub mod query;
pub mod rsp;

#[doc(inline)]
pub use self::query::{Query, Source, Step, Walk, WalkOrigin};

#[doc(inline)]
pub use self::rsp::{Edge, Response};

#[cfg(test)]
mod tests {
    use serde_json;

    use super::query::{EdgeAnnotation, Relationship, Role};
    use super::{Query, Source, Step, Walk};
    use crate::Oid;

    #[test]
    fn it_works() {
        let request = Query {
            from: 0.into(),
            walks: vec![Walk {
                origins: vec![Source::device(Oid::new(14))].into(),
                steps: vec![Step {
                    relationships: vec![Relationship::new("HTTP", Role::Server)],
                    ..Default::default()
                }],
            }],
            edge_annotations: vec![EdgeAnnotation::Protocols],
            ..Default::default()
        };

        println!("{}", serde_json::to_string_pretty(&request).unwrap());
    }
}
