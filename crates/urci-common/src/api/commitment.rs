//! Commitment response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Operator commitment (opt-in/opt-out to slasher)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorCommitmentResponse {
    // Slasher address
    #[cfg_attr(feature = "utoipa", schema(example = "0x123...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub slasher_address: String,

    // Committer address (optional)
    #[cfg_attr(feature = "utoipa", schema(nullable = true, example = "0x456...", pattern = "^0x[a-fA-F0-9]{40}$"))]
    pub committer_address: Option<String>,

    // When opted in
    #[cfg_attr(feature = "utoipa", schema(nullable = true))]
    pub opted_in_at: Option<String>,

    // When opted out
    #[cfg_attr(feature = "utoipa", schema(nullable = true))]
    pub opted_out_at: Option<String>,

    // Status (active, opted_out, slashed)
    #[cfg_attr(feature = "utoipa", schema(example = "active"))]
    pub status: String,

    // Event ID for this commitment
    pub event_id: Option<i64>,
}
