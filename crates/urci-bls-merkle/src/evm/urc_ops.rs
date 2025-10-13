//! UrcBlsMerkle trait implementation for EvmBlsMerkleExecutor
//!
//! Currently commented out - to be implemented

/*
use alloy_primitives::Address;
use eyre::Result;
use urci_common::{IRegistry, RegistrationValidationResult};

use super::executor::EvmBlsMerkleExecutor;
use crate::traits::UrcBlsMerkle;

impl UrcBlsMerkle for EvmBlsMerkleExecutor {
    fn implementation_type(&self) -> &'static str {
        "EVM"
    }

    fn verify_registration(
        &mut self,
        registrations: &[IRegistry::SignedRegistration],
        owner: Address,
    ) -> Result<RegistrationValidationResult> {
        use urci_common::{RegistrationSignatureValidation, MerkleTreeData};

        let mut validations = Vec::new();
        let mut has_fraudulent = false;

        // Use MerkleOps to hash registrations to leaves
        let leaves = self.hash_to_leaves(registrations, owner)?;

        // Generate the merkle tree root
        let root = self.generate_tree(&leaves)?;

        // Validate each registration
        for (index, registration) in registrations.iter().enumerate() {
            // Capture both the validation result and any error
            let (is_valid, validation_error) = match self.verify_signed_registration(registration, owner) {
                Ok(valid) => (valid, None),
                Err(e) => {
                    // Parse the error string into ValidationError enum
                    let parsed_error = urci_common::events::ValidationError::from_error_string(&e.to_string());
                    (false, Some(parsed_error))
                }
            };

            if !is_valid {
                has_fraudulent = true;
            }

            // Generate merkle proof
            let proof = self.generate_proof(&leaves, index)?;

            validations.push(RegistrationSignatureValidation {
                is_fraudulent: !is_valid,
                validation_error,
                signature_index: index,
                merkle_proof: proof,
                signature_data: registration.signature.clone(),
                pubkey_data: registration.pubkey.clone(),
            });
        }

        Ok(RegistrationValidationResult {
            has_fraudulent,
            tree: MerkleTreeData {
                root,
                leaves,
            },
            signature: validations,
        })
    }
}
*/