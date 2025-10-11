//! Verifier trait for BLS signature verification

use crate::{BLS, IRegistry};
use super::super::validation_result::RegistrationValidationResult;
use alloy_primitives::{Address, B256, U256};
use async_trait::async_trait;
use eyre::Result;

// Trait for BLS and Merkle operations
#[async_trait]
pub trait Verifier: Send + Sync {
    // Verify a batch of registrations (async)
    async fn verify_registration(
        &self,
        registrations: Vec<IRegistry::SignedRegistration>,
        owner: Address,
    ) -> Result<RegistrationValidationResult>;

    // Verify a batch of registrations (blocking)
    fn verify_registration_blocking(
        &self,
        registrations: Vec<IRegistry::SignedRegistration>,
        owner: Address,
    ) -> Result<RegistrationValidationResult>;

    // Verify a single signed registration (async)
    async fn verify_signed_registration(
        &self,
        registration: IRegistry::SignedRegistration,
        owner: Address,
    ) -> Result<bool>;

    // Verify a single signed registration (blocking)
    fn verify_signed_registration_blocking(
        &self,
        registration: IRegistry::SignedRegistration,
        owner: Address,
    ) -> Result<bool>;

    // Convert private key to public key
    async fn to_public_key(&self, private_key: U256) -> Result<BLS::G1Point>;

    // Sign a message
    async fn sign(
        &self,
        message: Vec<u8>,
        private_key: U256,
        domain_separator: Vec<u8>,
    ) -> Result<BLS::G2Point>;

    // Generate merkle tree root
    async fn generate_tree(&self, leaves: Vec<B256>) -> Result<B256>;

    // Generate merkle proof
    async fn generate_proof(&self, leaves: Vec<B256>, index: usize) -> Result<Vec<B256>>;
}
