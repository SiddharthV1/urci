use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Mainnet,
    Custom { chain_id: u64 },
}

impl Chain {
    pub const fn chain_id(&self) -> u64 {
        match self {
            Chain::Mainnet => 1,
            Chain::Custom { chain_id } => *chain_id,
        }
    }
}
