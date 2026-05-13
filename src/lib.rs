pub mod agency;
pub mod artifacts;
pub mod core;
pub mod energy;
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
