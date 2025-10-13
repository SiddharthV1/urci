//! MerkleOps trait implementation for CachedBlsExecutor

use alloy_primitives::{Address, B256};
use std::time::Instant;
use urci_common::IRegistry;
use crate::{BlsOps, MerkleOps};
use super::executor::CachedBlsExecutor;
use super::cache_keys::merkle_proof_cache_key;
use super::types::CachedMerkleProof;

// Implement MerkleOps with optional caching
impl<T> MerkleOps for CachedBlsExecutor<T>
where
    T: BlsOps + MerkleOps + Send + Sync,
{
    fn generate_tree(&mut self, leaves: &[B256]) -> eyre::Result<B256> {
        // Tree generation is fast, no need to cache
        self.inner.generate_tree(leaves)
    }

    fn generate_proof(&mut self, leaves: &[B256], index: usize) -> eyre::Result<Vec<B256>> {
        if !self.cache_merkle_proofs {
            return self.inner.generate_proof(leaves, index);
        }

        // Check cache
        let key = merkle_proof_cache_key(leaves, index);
        if let Some(cached) = self.merkle_cache.get(&key) {
            if Instant::now().duration_since(cached.timestamp) < self.ttl {
                return Ok(cached.proof.clone());
            }
        }

        // Generate and cache
        let proof = self.inner.generate_proof(leaves, index)?;
        let root = self.inner.generate_tree(leaves)?;

        self.merkle_cache.insert(key, CachedMerkleProof {
            proof: proof.clone(),
            root,
            timestamp: Instant::now(),
        });

        Ok(proof)
    }

    fn verify_proof(&mut self, root: B256, leaf: B256, proof: &[B256]) -> eyre::Result<bool> {
        // Verification is fast, no need to cache
        self.inner.verify_proof(root, leaf, proof)
    }

    fn hash_to_leaves(&mut self, registrations: &[IRegistry::SignedRegistration], owner: Address) -> eyre::Result<Vec<B256>> {
        // This is deterministic and fast, could cache but probably not worth it
        self.inner.hash_to_leaves(registrations, owner)
    }
}
