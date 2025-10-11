//! Timeline response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Timeline event for operator history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorTimelineEvent {
    // Event type
    pub event_type: String,

    // Block number
    pub block_number: i64,

    // Timestamp
    pub timestamp: Option<String>,

    // Event-specific data
    pub data: serde_json::Value,
}
