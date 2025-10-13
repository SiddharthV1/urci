//! BLS operations trait
//!
//! This trait matches the Solidity BLSUtils library interface
//! and can be implemented by both EVM-based and native implementations.

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolValue;
use eyre::Result;
use urci_common::{BLS, IRegistry};

/// Domain separators for URC
pub const URC_REGISTRATION_DOMAIN: &[u8] = b"\x00URC";
pub const URC_DELEGATION_DOMAIN: &[u8] = b"\x00Del";

/// BLS operations trait matching Solidity BLSUtils interface
pub trait BlsOps {
    /// Convert private key to public key (BLSUtils.toPublicKey)
    fn to_public_key(&mut self, private_key: U256) -> Result<BLS::G1Point>;

    /// Convert message to G2 point (BLSUtils.toMessagePoint)
    fn to_message_point(&mut self, message: &[u8], domain_separator: &[u8])
        -> Result<BLS::G2Point>;

    /// Sign a message (BLSUtils.sign)
    fn sign(
        &mut self,
        message: &[u8],
        private_key: U256,
        domain_separator: &[u8],
    ) -> Result<BLS::G2Point>;

    /// Verify a signature (BLSUtils.verify)
    fn verify(
        &mut self,
        message: &[u8],
        signature: &BLS::G2Point,
        public_key: &BLS::G1Point,
        domain_separator: &[u8],
    ) -> Result<bool>;

    /// URC-specific: Sign a registration
    fn sign_registration(
        &mut self,
        owner: Address,
        private_key: U256,
    ) -> Result<IRegistry::SignedRegistration> {
        // Default implementation using the base operations
        let pubkey = self.to_public_key(private_key)?;
        let message = owner.abi_encode();
        let signature = self.sign(&message, private_key, URC_REGISTRATION_DOMAIN)?;
        Ok(IRegistry::SignedRegistration { pubkey, signature })
    }

    /// URC-specific: Sign a delegation
    fn sign_delegation(&mut self, message: &[u8], private_key: U256) -> Result<BLS::G2Point> {
        self.sign(message, private_key, URC_DELEGATION_DOMAIN)
    }
}
