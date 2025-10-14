//! ExEx configuration

use alloy_primitives::Address;

/// Configuration for the URCI ExEx
#[derive(Debug, Clone)]
pub struct ExExConfig {
    pub registry_address: Address,
}

impl Default for ExExConfig {
    fn default() -> Self {
        Self {
            registry_address: alloy_primitives::address!("17435ccE3d1B4fA2e5f8A08eD921D57C6762A180"),
        }
    }
}
