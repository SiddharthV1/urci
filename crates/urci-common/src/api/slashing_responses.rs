//! Slashing response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Slashing event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SlashEventResponse {
    // Type of slash
    pub slash_type: String,

    // Event ID
    pub event_id: Option<i64>,

    // Block number
    pub block_number: i64,

    // Sender address (challenger/slasher)
    #[cfg_attr(feature = "utoipa", schema(example = "0x123...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub sender_address: String,

    // Sender reward in wei
    pub sender_reward_wei: String,

    // Amount burned in wei
    pub burned_wei: String,

    // Timestamp
    pub timestamp: Option<String>,
}

// Operator slashing history response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorSlashingResponse {
    // Total number of slashes
    pub total_slashes: i64,

    // Total amount burned across all slashes
    pub total_burned_wei: String,

    // List of slashing events
    pub slashes: Vec<SlashEventResponse>,
}

// Unified slashing event response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SlashingEventResponse {
    // Type of slash
    pub slash_type: String, // "registration", "commitment", "offchain_delegation", "equivocation"

    // Event ID
    pub event_id: Option<i64>,

    // Block number
    pub block_number: i64,

    // Registration root of slashed operator
    pub registration_root: String,

    // Sender address (challenger/slasher)
    #[cfg_attr(feature = "utoipa", schema(example = "0x123...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub sender_address: String,

    // Sender reward in wei
    pub sender_reward_wei: String,

    // Amount burned in wei
    pub burned_wei: String,

    // Timestamp
    pub timestamp: Option<String>,

    // Additional event-specific data
    pub details: serde_json::Value,
}

// Breakdown of slashes by type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SlashesByType {
    pub registration: i64,
    pub commitment: i64,
    pub offchain_delegation: i64,
    pub equivocation: i64,
}
