//! System errors that should flip indexer to failed state

use serde::{Deserialize, Serialize};

// System errors that should flip indexer to failed state
// These are not validation errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemError {
    // Failed to decode transaction calldata
    CalldataDecodeFailed(String),
    // EVM execution failed (unlinked contracts, out of gas, etc.)
    EvmExecutionFailed(String),
    // Failed to parse event data
    EventParseFailed(String),
    // Other system-level error
    Other(String),
}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemError::CalldataDecodeFailed(e) => write!(f, "Calldata decode failed: {}", e),
            SystemError::EvmExecutionFailed(e) => write!(f, "EVM execution failed: {}", e),
            SystemError::EventParseFailed(e) => write!(f, "Event parse failed: {}", e),
            SystemError::Other(e) => write!(f, "System error: {}", e),
        }
    }
}

impl std::error::Error for SystemError {}
