//! Committer response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Committer information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct CommitterResponse {
    // Committer address (20 bytes hex)
    #[cfg_attr(feature = "utoipa", schema(example = "0xdef...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub address: String,

    // Chain ID
    pub chain_id: i64,

    // Event ID where committer was first seen
    pub event_id: Option<i64>,

    // Created timestamp
    pub created_at: Option<String>,
}

// Slashing event with committer information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct CommitterSlashingEventResponse {
    // Type of slash
    pub slash_type: String, // "commitment", "offchain_delegation"

    // Event ID
    pub event_id: Option<i64>,

    // Block number
    pub block_number: i64,

    // Registration root of slashed operator
    pub registration_root: String,

    // Sender address (challenger)
    #[cfg_attr(feature = "utoipa", schema(example = "0x123...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub sender_address: String,

    // Committer address
    #[cfg_attr(feature = "utoipa", schema(example = "0x456...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub committer_address: String,

    // Sender reward in wei
    pub sender_reward_wei: String,

    // Amount burned in wei
    pub burned_wei: String,

    // Timestamp
    pub timestamp: Option<String>,

    // Additional event-specific data
    pub details: serde_json::Value,
}
