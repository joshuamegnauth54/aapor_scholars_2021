#![feature(entry_insert)]

//! # (Unofficial) Steam reviews API query builder
//!
//! `steam_review_api` builds queries for the [Steam reviews API](https://partner.steamgames.com/doc/store/getreviews).
//! This crate's code is _not_ affiliated with Valve or Steam in any way.
//!
//! I also provide convenience `struct`s for deserializing responses.
//!

mod buildapi;
mod error;
mod options;

// Convenience structs are feature gated in case someone does not want to use Serde.
#[cfg(feature = "convenience_structs")]
pub mod convenience_structs;

// Re-export the API builder, error enum, and options enums to ease importing.
pub use buildapi::ReviewApi;
pub use error::RevApiError;
pub use options::{Filter, PurchaseType, ReviewType};
