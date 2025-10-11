//! BLS key response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// BLS key with registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct BlsKeyResponse {
    // BLS G1 public key as hex
    #[cfg_attr(feature = "utoipa", schema(example = "0x123..."))]
    pub pubkey_hex: String,

    // Index in the registration merkle tree
    pub key_index: Option<i32>,

    // Merkle proof (array of 32-byte hashes)
    pub merkle_proof: Option<Vec<String>>,
}

// Operator keys response (for /operators/{chain_id}/{registration_root}/keys)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorKeysResponse {
    // Registration root
    #[cfg_attr(feature = "utoipa", schema(example = "0x123..."))]
    pub registration_root: String,

    // Chain ID
    pub chain_id: i64,

    // Number of keys
    pub num_keys: i32,

    // Array of BLS keys with proofs
    pub keys: Vec<BlsKeyResponse>,
}
