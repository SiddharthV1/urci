//! Merkle tree operations trait
//!
//! This trait matches the Solidity MerkleTree library interface
//! and can be implemented by both EVM-based and native implementations.

use alloy_primitives::{Address, B256};
use eyre::Result;
use urci_common::IRegistry;

/// Merkle tree operations trait matching Solidity MerkleTree interface
pub trait MerkleOps {
    /// Generate merkle tree root (MerkleTree.generateTree)
    fn generate_tree(&mut self, leaves: &[B256]) -> Result<B256>;

    /// Generate merkle proof (MerkleTree.generateProof)
    fn generate_proof(&mut self, leaves: &[B256], index: usize) -> Result<Vec<B256>>;

    /// Verify a merkle proof (MerkleTree.verifyProof)
    fn verify_proof(&mut self, root: B256, leaf: B256, proof: &[B256]) -> Result<bool>;

    /// Hash registrations to leaves (MerkleTree.hashToLeaves)
    fn hash_to_leaves(
        &mut self,
        registrations: &[IRegistry::SignedRegistration],
        owner: Address,
    ) -> Result<Vec<B256>>;
}
