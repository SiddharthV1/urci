//! Cached BLS executor wrapper
//!
//! Wraps any UrcBlsMerkle executor with caching for:
//! 1. BLS signature validations (per registration + owner)
//! 2. Merkle proof generation (for fraud proofs)
//! 3. Complete registration validation results

mod bls_ops;
mod cache_keys;
mod executor;
mod merkle_ops;
mod types;
mod urc_ops;

// Re-export public types and functions
pub use cache_keys::{bls_cache_key, fraud_proof_cache_key, merkle_proof_cache_key};
pub use executor::CachedBlsExecutor;
pub use types::{CachedBLSValidation, CachedMerkleProof, CachedRegistrationValidation};
