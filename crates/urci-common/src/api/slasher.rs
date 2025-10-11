//! Slasher response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use super::operator::OperatorResponse;
use super::slashing_responses::SlashesByType;

// Slasher information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SlasherResponse {
    // Slasher address (20 bytes hex)
    #[cfg_attr(feature = "utoipa", schema(example = "0xabc...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub address: String,

    // Chain ID
    pub chain_id: i64,

    // Event ID where slasher was first seen
    pub event_id: Option<i64>,

    // Created timestamp
    pub created_at: Option<String>,
}

// Operator with commitment status to a slasher
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorWithCommitmentResponse {
    // Base operator information
    #[serde(flatten)]
    pub operator: OperatorResponse,

    // Commitment status
    pub commitment_status: String, // "active", "opted_out"

    // When opted in
    pub opted_in_at: Option<String>,

    // When opted out (if applicable)
    pub opted_out_at: Option<String>,

    // Committer address (if any)
    pub committer_address: Option<String>,

    // Whether this operator has been slashed by this slasher
    pub slashed: bool,
}

// Slasher statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SlasherStatsResponse {
    // Slasher address
    #[cfg_attr(feature = "utoipa", schema(example = "0xabc...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub slasher_address: String,

    // Chain ID
    pub chain_id: i64,

    // Total operators ever committed
    pub total_operators: i64,

    // Currently active operators (not opted out)
    pub active_operators: i64,

    // Total slashing events
    pub total_slashes: i64,

    // Slashes by type
    pub slashes_by_type: SlashesByType,

    // Total rewards distributed (wei)
    pub total_rewards_wei: String,

    // Total collateral burned (wei)
    pub total_burned_wei: String,
}
