use super::cache_keys::bls_cache_key;
use super::executor::CachedBlsExecutor;
use super::types::{CachedBLSValidation, CachedRegistrationValidation};
use crate::traits::URC_REGISTRATION_DOMAIN;
use crate::{BlsOps, MerkleOps, UrcBlsMerkle};
use alloy_primitives::{keccak256, Address};
use alloy_sol_types::SolValue;
use eyre::Result;
use std::time::Instant;
use urci_common::{
    IRegistry, MerkleTreeData, RegistrationSignatureValidation, RegistrationValidationResult,
};

// Implement UrcBlsMerkle with caching for verify_signed_registration
impl<T> UrcBlsMerkle for CachedBlsExecutor<T>
where
    T: BlsOps + MerkleOps + Send + Sync,
{
    fn implementation_type(&self) -> &'static str {
        "cached"
    }

    fn verify_signed_registration(
        &mut self,
        registration: &IRegistry::SignedRegistration,
        owner: Address,
    ) -> eyre::Result<bool> {
        // Generate cache key
        let key = bls_cache_key(&registration.pubkey, &registration.signature, owner);

        // Check cache
        if let Some(cached) = self.bls_cache.get(&key) {
            if Instant::now().duration_since(cached.timestamp) < self.ttl {
                // Cache hit
                return if cached.is_valid {
                    Ok(true)
                } else {
                    Err(eyre::eyre!(
                        "{}",
                        cached
                            .validation_error
                            .as_deref()
                            .unwrap_or("Invalid BLS signature")
                    ))
                };
            }
        }

        // Cache miss or expired - validate using BlsOps
        let message = owner.abi_encode();
        let result = self.inner.verify(
            &message,
            &registration.signature,
            &registration.pubkey,
            URC_REGISTRATION_DOMAIN,
        );

        // Cache the result
        let validation = CachedBLSValidation {
            is_valid: result.is_ok(),
            validation_error: result.as_ref().err().map(|e| e.to_string()),
            timestamp: Instant::now(),
        };
        self.bls_cache.insert(key, validation);

        // Periodic cleanup
        if self.bls_cache.len() > self.max_entries_per_cache * 2 {
            self.cleanup();
        }

        result
    }

    fn verify_registration(
        &mut self,
        registrations: &[IRegistry::SignedRegistration],
        owner: Address,
    ) -> Result<RegistrationValidationResult> {
        // Generate cache key using ABI encoding
        let cache_key = keccak256((registrations.to_vec(), owner).abi_encode_packed());

        // Check registration cache first
        if let Some(cached) = self.registration_cache.get(&cache_key) {
            if Instant::now().duration_since(cached.timestamp) < self.ttl {
                // Cache hit - return the cached result
                return Ok(cached.result.clone());
            }
        }

        // Cache miss - compute the full validation
        let mut validations = Vec::new();
        let mut has_fraudulent = false;

        // Use MerkleOps to hash registrations to leaves (exactly as the contract does)
        let leaves = self.hash_to_leaves(registrations, owner)?;

        // Generate the merkle tree root using MerkleOps
        let root = self.generate_tree(&leaves)?;

        // Validate each registration and generate proofs
        for (index, registration) in registrations.iter().enumerate() {
            // Validate BLS signature using verify_signed_registration (which uses cache)
            let validation_result = self.verify_signed_registration(registration, owner);

            let is_valid = validation_result.is_ok();
            let validation_error = validation_result
                .err()
                .map(|e| urci_common::ValidationError::from_error_string(&e.to_string()));

            if !is_valid {
                has_fraudulent = true;
            }

            // Generate merkle proof for this index using MerkleOps
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

        let result = RegistrationValidationResult {
            has_fraudulent,
            tree: MerkleTreeData { root, leaves },
            signature: validations,
        };

        // Cache the result
        self.registration_cache.insert(
            cache_key,
            CachedRegistrationValidation {
                result: result.clone(),
                timestamp: Instant::now(),
            },
        );

        // Periodic cleanup
        if self.registration_cache.len() > self.max_entries_per_cache * 2 {
            self.cleanup();
        }

        Ok(result)
    }
}
