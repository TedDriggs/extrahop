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
extern crate derive_builder;

#[macro_use]
extern crate error_chain;

extern crate reqwest;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
mod macros;

mod api_key;
mod client;
mod oid;
mod traits;
mod user;
mod user_group;

pub mod activitymap;
pub mod errors;
pub mod sharing;

pub use api_key::ApiKey;
pub use client::Client;
pub use errors::{Error, ErrorKind, Result, ResultExt};
pub use oid::Oid;
pub use traits::{ApiResponse, Patch};
pub use user::Username;
pub use user_group::UserGroupId;