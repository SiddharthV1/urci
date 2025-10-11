//! Fork detection response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Fork detection status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ForkStatusResponse {
    // Chain ID
    pub chain_id: i64,

    // Highest block number all synced writers agree on
    pub highest_common_block: i64,

    // Whether any forks have been detected
    pub forks_detected: bool,

    // Different block heads at the highest common block (if forked)
    pub fork_heads: Vec<ForkHeadResponse>,

    // Number of synced writers
    pub synced_writers: i32,

    // Total number of active writers
    pub total_writers: i32,
}

// Fork head information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ForkHeadResponse {
    // Block number
    pub block_number: i64,

    // Block hash
    #[cfg_attr(feature = "utoipa", schema(example = "0xaaa..."))]
    pub block_hash: String,

    // Last event ID for this fork head
    pub head_event_id: Option<i64>,

    // Whether this block is canonical
    pub canonical: bool,

    // Number of writers seeing this fork
    pub writer_count: i32,
}
