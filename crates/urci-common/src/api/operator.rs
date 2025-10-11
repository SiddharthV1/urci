//! Operator response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use super::commitment::OperatorCommitmentResponse;
use super::slashing_responses::SlashEventResponse;
use super::timeline::OperatorTimelineEvent;
use super::bls_key::BlsKeyResponse;

// Operator summary (for list endpoints)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorResponse {
    // Registration root (32 bytes hex)
    #[cfg_attr(feature = "utoipa", schema(example = "0x123...", pattern = "^0x[a-fA-F0-9]{64}$"))]
    pub registration_root: String,

    // Chain ID
    pub chain_id: i64,

    // Owner address (20 bytes hex)
    #[cfg_attr(feature = "utoipa", schema(example = "0xabc...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub owner_address: String,

    // Number of BLS keys registered
    pub num_keys: Option<i32>,

    // Whether registration has been processed
    pub registration_processed: bool,

    // Block number where this state applies
    pub block_number: Option<i64>,

    // Total collateral in wei (as string to handle uint256)
    #[cfg_attr(feature = "utoipa", schema(nullable = true, example = "1000000000000000000"))]
    pub collateral_wei_total: Option<String>,

    // Last event type
    pub last_event_type: Option<String>,

    // Whether operator is deleted
    pub deleted: Option<bool>,

    // Whether operator equivocated
    pub equivocated: Option<bool>,

    // Whether block is canonical
    pub canonical: Option<bool>,

    // Whether block is finalized
    pub finalized: Option<bool>,

    // Sync status
    #[cfg_attr(feature = "utoipa", schema(example = "finalized"))]
    pub sync_status: Option<String>,

    // All slasher commitments (only included in detail endpoint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitments: Option<Vec<OperatorCommitmentResponse>>,

    // Complete slashing history (only included in detail endpoint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slashing_history: Option<Vec<SlashEventResponse>>,

    // Chronological timeline of all events (only included in detail endpoint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<Vec<OperatorTimelineEvent>>,
}

// Operator search result with keys
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorSearchResponse {
    // Operator information
    pub operator: OperatorResponse,

    // BLS keys registered by this operator
    pub keys: Vec<BlsKeyResponse>,
}

// Operator with pubkey index (for by_pubkey query results)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorWithPubkeyIndex {
    // Registration root
    #[cfg_attr(feature = "utoipa", schema(example = "0x123..."))]
    pub registration_root: String,

    // Chain ID
    pub chain_id: i64,

    // Owner address
    #[cfg_attr(feature = "utoipa", schema(example = "0xabc..."))]
    pub owner_address: String,

    // Index where this pubkey appears in operator's key list
    pub key_index: i32,
}
