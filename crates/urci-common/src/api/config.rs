//! Contract configuration response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Contract configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct ContractConfigResponse {
    // URC Registry contract address
    #[cfg_attr(feature = "utoipa", schema(example = "0x17435ccE3d1B4fA2e5f8A08eD921D57C6762A180"))]
    pub registry_address: String,

    // Contract configuration parameters
    pub config: RegistryConfig,
}

// Registry contract configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct RegistryConfig {
    // Minimum collateral required in wei
    #[cfg_attr(feature = "utoipa", schema(example = "1000000000000000000"))]
    pub min_collateral_wei: String,

    // Fraud proof window in blocks
    #[cfg_attr(feature = "utoipa", schema(example = 10))]
    pub fraud_proof_window_blocks: i64,

    // Unregistration delay in blocks
    #[cfg_attr(feature = "utoipa", schema(example = 5))]
    pub unregistration_delay_blocks: i64,

    // Slashing window in blocks
    #[cfg_attr(feature = "utoipa", schema(example = 20))]
    pub slashing_window_blocks: i64,

    // Opt-in delay in blocks
    #[cfg_attr(feature = "utoipa", schema(example = 3))]
    pub opt_in_delay_blocks: i64,
}
