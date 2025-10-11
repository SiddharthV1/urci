//! Common types and utilities shared across all URC indexer components.
//!
//! This crate provides the foundational types, traits, and utilities used throughout
//! the indexer. It follows the Commit-Boost pattern of having a dedicated common crate
//! for shared functionality.

pub mod api;
pub mod bindings;
pub mod config;
pub mod contracts;
pub mod core;
pub mod metrics;
pub mod sqlx_support;
pub mod tracing_setup;

// Re-exports for convenience
pub use api::*;
pub use config::*;
pub use contracts::*;
pub use core::*;
pub use metrics::*;
