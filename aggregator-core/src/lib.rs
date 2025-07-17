//! Core types and traits for cryptocurrency orderbook aggregation

pub mod aggregator;
pub mod config;
pub mod error;
pub mod types;

pub use aggregator::*;
pub use config::*;
pub use error::*;
pub use types::*;
