//! Merkle tree data structures

use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTreeData {
    pub root: B256,
    pub leaves: Vec<B256>,
}
