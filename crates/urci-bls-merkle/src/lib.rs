//! EVM-based BLS and Merkle operations for testing and fallback
//!
//! This crate serves two critical purposes:
//! A. Generate test data from actual EVM execution to verify urc-proof-lib
//! B. Provide a fallback implementation when contract changes haven't been reflected in urc-proof-lib
//!
//! It deploys wrapper contracts for BLSUtils and MerkleTree libraries in an EVM environment
//! and executes them to get exact URC-compatible results.

pub mod cached_executor;
pub mod evm;
pub mod handle;
pub mod precompiled;
pub mod traits;
pub mod wrapper_contracts;

use alloy_primitives::Address;

// Re-export main types
pub use cached_executor::CachedBlsExecutor;
pub use handle::{launch_bls_executor, BlsMerkleHandle};
pub use traits::{create_executor, BlsOps, ExecutorType, MerkleOps, UrcBlsMerkle};

// Re-export Solidity types from urc-common bindings
pub use urci_common::bindings::MerkleTree;
pub use urci_common::bindings::BLS;

/// Contract addresses for wrapper contracts deployed in genesis
pub const BLS_WRAPPER_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xB1, 0x55,
]);
pub const MERKLE_WRAPPER_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xAE, 0x33,
]);
/// BLSUtils library address - must match what's used in contract linking
pub const BLS_UTILS_LIB_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xB1, 0x56,
]);

/// Convenience function to create an EVM executor directly
pub fn create_evm_executor() -> eyre::Result<Box<dyn UrcBlsMerkle + Send + Sync>> {
    create_executor(ExecutorType::Evm)
}
