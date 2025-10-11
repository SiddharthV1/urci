//! Block response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use super::event::EventResponse;

// Block data with URC events (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct BlockResponse {
    // Chain ID
    pub chain_id: i64,

    // Block number
    pub number: i64,

    // Block hash (0x-prefixed hex)
    #[cfg_attr(feature = "utoipa", schema(example = "0xabc123...", pattern = "^0x[a-fA-F0-9]{64}$"))]
    pub hash: String,

    // Parent block hash (0x-prefixed hex)
    #[cfg_attr(feature = "utoipa", schema(example = "0xdef456...", pattern = "^0x[a-fA-F0-9]{64}$"))]
    pub parent_hash: String,

    // Work ID - computed hash of (chain_id, number, hash, parent_work_id)
    #[cfg_attr(feature = "utoipa", schema(example = "0x789abc...", pattern = "^0x[a-fA-F0-9]{64}$"))]
    pub work_id: String,

    // Parent block's work_id (null for genesis)
    #[cfg_attr(feature = "utoipa", schema(nullable = true, example = "0x456def...", pattern = "^0x[a-fA-F0-9]{64}$"))]
    pub parent_work_id: Option<String>,

    // Block timestamp
    pub timestamp: i64,

    // Whether this block is on the canonical chain
    pub canonical: bool,

    // Whether this block has been finalized
    pub finalized: bool,

    // The ID of the last event for this block (used for fork detection)
    #[cfg_attr(feature = "utoipa", schema(nullable = true))]
    pub head_event_id: Option<i64>,

    // Total number of URC events in this block
    pub event_count: i64,

    // All URC events that occurred in this block
    pub events: Vec<EventResponse>,

    // ID of the writer that indexed this block
    #[cfg_attr(feature = "utoipa", schema(format = "uuid"))]
    pub writer_id: Option<String>,
}
