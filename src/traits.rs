use serde::Serialize;

/// Marker trait for types that are meant to use in PATCH requests.
pub trait Patch: Default + Serialize {}
