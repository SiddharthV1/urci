//! API types shared between urc-api and urc-db-adapter
//!
//! These types represent the data structures returned by the API
//! and constructed by the database adapter's reader methods.

pub mod block;
pub mod event;
pub mod operator;
pub mod fork;
pub mod pagination;
pub mod bls_key;
pub mod collateral;
pub mod commitment;
pub mod slashing_responses;
pub mod slasher;
pub mod committer;
pub mod timeline;
pub mod config;

// Re-exports for convenience
pub use block::*;
pub use event::*;
pub use operator::*;
pub use fork::*;
pub use pagination::*;
pub use bls_key::*;
pub use collateral::*;
pub use commitment::*;
pub use slashing_responses::*;
pub use slasher::*;
pub use committer::*;
pub use timeline::*;
pub use config::*;
