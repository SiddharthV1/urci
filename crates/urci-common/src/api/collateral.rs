//! Collateral response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Collateral entry for history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct CollateralEntry {
    // Collateral total at this point
    pub collateral_wei_total: String,

    // Delta (change amount)
    pub collateral_wei_delta: Option<String>,

    // Event ID
    pub event_id: Option<i64>,

    // Timestamp
    pub created_at: Option<String>,
}

// Operator collateral response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct OperatorCollateralResponse {
    // Current total collateral
    pub current_total: String,

    // History of collateral changes
    pub history: Vec<CollateralEntry>,
}

// Total collateral statistics for a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct TotalCollateralResponse {
    // Chain ID
    pub chain_id: i64,

    // Total collateral locked in the system (wei)
    #[cfg_attr(feature = "utoipa", schema(example = "1000000000000000000"))]
    pub total_collateral_wei: String,

    // Total number of operators
    pub total_operators: i64,

    // Number of operators with non-zero collateral
    pub operators_with_collateral: i64,

    // Average collateral per operator (wei)
    #[cfg_attr(feature = "utoipa", schema(example = "500000000000000000"))]
    pub average_collateral_wei: String,
}
