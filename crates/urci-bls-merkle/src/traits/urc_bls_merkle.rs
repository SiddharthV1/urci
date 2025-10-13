//! Combined URC BLS and Merkle operations trait
//!
//! This trait combines both BLS and Merkle operations for URC-specific use cases.
//! Implementations can provide either EVM-based or native execution.

use alloy_primitives::Address;
use alloy_sol_types::SolValue;
use eyre::Result;
use urci_common::{IRegistry, RegistrationValidationResult};

use super::bls_ops::{BlsOps, URC_REGISTRATION_DOMAIN};
use super::merkle_ops::MerkleOps;

/// Combined URC BLS and Merkle operations trait
///
/// This trait combines both BLS and Merkle operations for URC-specific use cases.
/// Implementations can provide either EVM-based or native execution.
pub trait UrcBlsMerkle: BlsOps + MerkleOps {
    /// Get the implementation type for debugging/logging
    fn implementation_type(&self) -> &'static str;

    /// Verify a signed registration (combines BLS verify with URC logic)
    fn verify_signed_registration(
        &mut self,
        registration: &IRegistry::SignedRegistration,
        owner: Address,
    ) -> Result<bool> {
        let message = owner.abi_encode();
        self.verify(
            &message,
            &registration.signature,
            &registration.pubkey,
            URC_REGISTRATION_DOMAIN,
        )
    }

    fn verify_registration(
        &mut self,
        registration: &[IRegistry::SignedRegistration],
        owner: Address,
    ) -> Result<RegistrationValidationResult>;
}
