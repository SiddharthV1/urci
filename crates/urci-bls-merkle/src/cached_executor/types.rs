//! Cache data structures

use alloy_primitives::B256;
use std::time::Instant;
use urci_common::RegistrationValidationResult;

/// Cached BLS validation result
#[derive(Debug, Clone)]
pub struct CachedBLSValidation {
    pub is_valid: bool,
    pub validation_error: Option<String>,
    pub timestamp: Instant,
}

/// Cached merkle proof
#[derive(Debug, Clone)]
pub struct CachedMerkleProof {
    pub proof: Vec<B256>,
    pub root: B256,
    pub timestamp: Instant,
}

/// Cached registration validation result
#[derive(Debug, Clone)]
pub struct CachedRegistrationValidation {
    pub result: RegistrationValidationResult,
    pub timestamp: Instant,
}
