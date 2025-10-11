//! Event response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// URC event (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct EventResponse {
    // Event ID
    pub id: i64,

    // Chain ID
    pub chain_id: i64,

    // Block number
    pub block_number: i64,

    // Block hash
    #[cfg_attr(feature = "utoipa", schema(example = "0xabc..."))]
    pub block_hash: String,

    // Transaction hash
    #[cfg_attr(feature = "utoipa", schema(example = "0xdef..."))]
    pub tx_hash: String,

    // Transaction index in block
    pub tx_index: i32,

    // Log index in block
    pub log_index: i32,

    // Event type
    #[cfg_attr(feature = "utoipa", schema(example = "OperatorRegistered"))]
    pub event_type: String,

    // Event-specific decoded data
    pub decoded_data: serde_json::Value,

    // Writer ID
    #[cfg_attr(feature = "utoipa", schema(format = "uuid"))]
    pub writer_id: Option<String>,
}
