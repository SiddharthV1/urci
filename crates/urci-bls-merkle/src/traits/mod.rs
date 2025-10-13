//! Trait definitions for BLS and Merkle operations
//!
//! These traits match the Solidity BLSUtils and MerkleTree library interfaces
//! and can be implemented by both EVM-based and native implementations.

mod bls_ops;
mod merkle_ops;
mod urc_bls_merkle;
mod factory;

// Re-export all public types
pub use bls_ops::{BlsOps, URC_DELEGATION_DOMAIN, URC_REGISTRATION_DOMAIN};
pub use merkle_ops::MerkleOps;
pub use urc_bls_merkle::UrcBlsMerkle;
pub use factory::{create_executor, ExecutorType};
