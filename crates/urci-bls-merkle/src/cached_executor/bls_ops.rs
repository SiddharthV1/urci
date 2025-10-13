//! BlsOps trait implementation for CachedBlsExecutor

use alloy_primitives::U256;
use urci_common::BLS;
use crate::BlsOps;
use super::executor::CachedBlsExecutor;
use crate::MerkleOps;

// Implement BlsOps - most methods just forward to inner
impl<T> BlsOps for CachedBlsExecutor<T>
where
    T: BlsOps + MerkleOps + Send + Sync,
{
    fn to_public_key(&mut self, private_key: U256) -> eyre::Result<BLS::G1Point> {
        self.inner.to_public_key(private_key)
    }

    fn to_message_point(&mut self, message: &[u8], domain_separator: &[u8]) -> eyre::Result<BLS::G2Point> {
        self.inner.to_message_point(message, domain_separator)
    }

    fn sign(&mut self, message: &[u8], private_key: U256, domain_separator: &[u8]) -> eyre::Result<BLS::G2Point> {
        self.inner.sign(message, private_key, domain_separator)
    }

    fn verify(
        &mut self,
        message: &[u8],
        signature: &BLS::G2Point,
        public_key: &BLS::G1Point,
        domain_separator: &[u8],
    ) -> eyre::Result<bool> {
        // For general verify without owner context, we don't cache
        self.inner.verify(message, signature, public_key, domain_separator)
    }
}
