//! ExtraHop REST API client.
//!
//! This crate provides an asynchronous client for working with the REST API, along with
//! some feature-gated stronger types for scenarios such as working with activity maps.
//! It is not a goal of this crate to provide complete API bindings, as doing so would
//! make the crate dependent on specific firmware versions.
//!
//! # Getting Started
//! Appliances using self-signed SSL certificates will get an error using this library
//! because the host OS can't establish a secure connection. To address this, get the
//! public certificate from `http://{EXTRAHOP_HOST}/public.cer` and trust it at the system
//! level.

mod api_response;
pub mod client;
mod error;
mod oid;
mod query_time;

#[cfg(feature = "topology")]
pub mod activitymap;

pub use api_response::ApiResponse;
#[doc(inline)]
pub use client::{CertVerification, Client};
pub use error::{Error, RestError};
pub use oid::Oid;
pub use query_time::QueryTime;

pub type Result<T> = std::result::Result<T, Error>;
