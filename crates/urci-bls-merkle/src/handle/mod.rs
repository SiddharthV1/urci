//! Unified handle for communicating with the BLS executor thread
//!
//! This module provides both sync and async interfaces to the BLS/Merkle executor
//! that runs in a separate thread. All communication happens via channels.

mod types;
mod client;
mod worker;
mod launcher;
mod verifier;

// Re-export public types and functions
pub use types::EvmThreadRequest;
pub use client::BlsMerkleHandle;
pub use launcher::launch_bls_executor;
