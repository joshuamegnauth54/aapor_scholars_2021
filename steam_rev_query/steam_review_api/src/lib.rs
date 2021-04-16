#![feature(entry_insert)]

//! # (Unofficial) Steam reviews API query builder
//!
//! `steam_review_api` builds queries for the [Steam reviews API](https://partner.steamgames.com/doc/store/getreviews).
//! This crate's code is _not_ affiliated with Valve or Steam in any way.
//!
//! I also provide convenience `struct`s for deserializing responses via Serde.
//! [conveniencestructs::flat_query::FlattenedQuery] is a flattened version of the response with
//! some of the useful data pulled out.
//! Enable the `convenience_structs` feature if and of that sounds remotely useful. 😸
//!

mod buildapi;
mod error;
mod language;
mod options;

// Convenience structs are feature gated in case someone does not want to use Serde.
#[cfg(feature = "convenience_structs")]
pub mod convenience_structs;

// Re-export the API builder, error enum, and options enums to ease importing.
pub use buildapi::ReviewApi;
pub use error::RevApiError;
pub use language::Language;
pub use options::{Filter, PurchaseType, ReviewType};
