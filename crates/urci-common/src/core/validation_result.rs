//! Registration validation results and signature validation

use alloy_primitives::FixedBytes;
use serde::{Deserialize, Serialize};

use super::merkle::MerkleTreeData;
use super::validation_error::ValidationError;

// Result of fraud detection - contains everything needed for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationValidationResult {
    pub has_fraudulent: bool,
    pub tree: MerkleTreeData,
    pub signature: Vec<RegistrationSignatureValidation>, // Multiple fraud proofs for multiple invalid registrations
}

impl RegistrationValidationResult {
    pub fn number_of_keys(&self) -> usize {
        self.signature.len()
    }
}

pub type BlsPubKey = crate::BLS::G1Point;
pub type BlsSignature = crate::BLS::G2Point;

// Complete fraud proof ready for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationSignatureValidation {
    pub is_fraudulent: bool,
    pub validation_error: Option<ValidationError>,
    pub signature_index: usize,
    pub merkle_proof: Vec<FixedBytes<32>>,
    pub signature_data: BlsSignature,
    pub pubkey_data: BlsPubKey,
}
