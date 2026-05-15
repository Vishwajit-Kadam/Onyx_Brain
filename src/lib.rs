//! Onyx Brain — crate root.
//!
//! This file is intentionally small: it re-exports all top-level modules and
//! the `Brain` facade. All logic lives in the respective submodules.
pub mod agency;
pub mod app_api;
pub mod artifacts;
pub mod conversation;
pub mod core;
pub mod creative;
pub mod energy;
pub mod executive;
pub mod experts;
pub mod learning;
pub mod memory;
pub mod routing;
pub mod sleep;
pub mod storage;
pub mod testing;
pub mod tools;
pub mod utils;

pub const ONYX_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));
pub const ONYX_VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");

pub use crate::core::brain::Brain;
