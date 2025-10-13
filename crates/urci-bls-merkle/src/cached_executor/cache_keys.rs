//! Cache key generation functions

use alloy_primitives::{Address, B256, keccak256};
use urci_common::BLS;

/// Generate cache key for BLS validation
/// Key = hash(pubkey + signature + owner)
pub fn bls_cache_key(
    pubkey: &BLS::G1Point,
    signature: &BLS::G2Point,
    owner: Address,
) -> B256 {
    let mut data = Vec::new();

    // Add pubkey components (G1Point has x_a, x_b, y_a, y_b)
    data.extend_from_slice(&pubkey.x_a.0);
    data.extend_from_slice(&pubkey.x_b.0);
    data.extend_from_slice(&pubkey.y_a.0);
    data.extend_from_slice(&pubkey.y_b.0);

    // Add signature components (G2Point has x_c0_a, x_c0_b, x_c1_a, x_c1_b, y_c0_a, y_c0_b, y_c1_a, y_c1_b)
    data.extend_from_slice(&signature.x_c0_a.0);
    data.extend_from_slice(&signature.x_c0_b.0);
    data.extend_from_slice(&signature.x_c1_a.0);
    data.extend_from_slice(&signature.x_c1_b.0);
    data.extend_from_slice(&signature.y_c0_a.0);
    data.extend_from_slice(&signature.y_c0_b.0);
    data.extend_from_slice(&signature.y_c1_a.0);
    data.extend_from_slice(&signature.y_c1_b.0);

    // Add owner (critical - same registration can be valid for one owner but not another)
    data.extend_from_slice(owner.as_slice());

    keccak256(&data)
}

/// Generate cache key for merkle proof
/// Key = hash(leaves + index)
pub fn merkle_proof_cache_key(leaves: &[B256], index: usize) -> B256 {
    let mut data = Vec::new();
    for leaf in leaves {
        data.extend_from_slice(leaf.as_slice());
    }
    data.extend_from_slice(&index.to_le_bytes());
    keccak256(&data)
}

/// Generate cache key for complete fraud proof
/// Key = hash(registration_root + registration_index + owner)
pub fn fraud_proof_cache_key(
    registration_root: B256,
    registration_index: usize,
    owner: Address,
) -> B256 {
    let mut data = Vec::new();
    data.extend_from_slice(registration_root.as_slice());
    data.extend_from_slice(&registration_index.to_le_bytes());
    data.extend_from_slice(owner.as_slice());
    keccak256(&data)
}
