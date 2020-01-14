//! ExtraHop REST API client.
//!
//! This client provides utility types for making general API requests, and strongly-typed
//! objects for metrics requests and other specific scenarios. This client is designed for
//! use with `reqwest` and `serde`.
//!
//! # Getting Started
//! Appliances using self-signed SSL certificates will get an error using this library
//! because the host OS can't establish a secure connection. To address this, get the
//! public certificate from `http://{EXTRAHOP_HOST}/public.cer` and trust it at the system
//! level.

#[macro_use]
mod macros;

mod api_response;
mod client;
mod error;
mod oid;
mod query_time;
mod traits;
mod user;
mod user_group;

#[cfg(feature = "topology")]
pub mod activitymap;
#[cfg(feature = "sharing")]
pub mod sharing;

pub use api_response::ApiResponse;
pub use client::Client;
pub use error::{Error, RestError};
pub use oid::Oid;
pub use query_time::QueryTime;
pub use traits::Patch;
pub use user::Username;
pub use user_group::UserGroupId;

pub type Result<T> = std::result::Result<T, Error>;
