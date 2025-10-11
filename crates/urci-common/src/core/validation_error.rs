use serde::{Deserialize, Serialize};

// Validation errors from BLS and Merkle Solidity libraries
// These are EXPECTED errors from on-chain verification and should be indexed normally
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationError {
    // BLS precompile errors (from BLS.sol)
    BlsPairingFailed,
    BlsG1AddFailed,
    BlsG1MSMFailed,
    BlsG2AddFailed,
    BlsG2MSMFailed,
    BlsMapFpToG1Failed,
    BlsMapFp2ToG2Failed,

    // Merkle library errors (from MerkleTree.sol)
    MerkleEmptyLeaves,
    MerkleIndexOutOfBounds,
    MerkleLeavesTooLarge,

    // EVM execution errors (non-Solidity)
    EvmExecutionFailed(String),
}

impl ValidationError {
    // Parse error string from EVM executor into ValidationError enum
    pub fn from_error_string(error: &str) -> Self {
        if error.contains("BLS: PairingFailed") || error.contains("PairingFailed") {
            Self::BlsPairingFailed
        } else if error.contains("BLS: G1AddFailed") || error.contains("G1AddFailed") {
            Self::BlsG1AddFailed
        } else if error.contains("BLS: G1MSMFailed") || error.contains("G1MSMFailed") {
            Self::BlsG1MSMFailed
        } else if error.contains("BLS: G2AddFailed") || error.contains("G2AddFailed") {
            Self::BlsG2AddFailed
        } else if error.contains("BLS: G2MSMFailed") || error.contains("G2MSMFailed") {
            Self::BlsG2MSMFailed
        } else if error.contains("BLS: MapFpToG1Failed") || error.contains("MapFpToG1Failed") {
            Self::BlsMapFpToG1Failed
        } else if error.contains("BLS: MapFp2ToG2Failed") || error.contains("MapFp2ToG2Failed") {
            Self::BlsMapFp2ToG2Failed
        } else if error.contains("Merkle: EmptyLeaves") || error.contains("EmptyLeaves") {
            Self::MerkleEmptyLeaves
        } else if error.contains("Merkle: IndexOutOfBounds") || error.contains("IndexOutOfBounds") {
            Self::MerkleIndexOutOfBounds
        } else if error.contains("Merkle: LeavesTooLarge") || error.contains("LeavesTooLarge") {
            Self::MerkleLeavesTooLarge
        } else {
            Self::EvmExecutionFailed(error.to_string())
        }
    }
}
