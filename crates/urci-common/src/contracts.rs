use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

sol!(
    #[derive(Debug, Deserialize, Serialize)]
    Registry,
    "registry_abi.json"
);

pub use Registry::*;
