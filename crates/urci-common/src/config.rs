//! Configuration module for URC indexer components.
//!
//! Configuration comes from multiple sources:
//! - Node config: From reth's CLI parser (chain, network, etc.)
//! - User & DB config: From a single TOML config file

use alloy_primitives::Address;
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Main URCI configuration combining all config sources
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UrciConfig {
    // Node configuration (filled from reth CLI)
    #[serde(skip)]
    pub node: NodeConfig,

    // Database configuration (from TOML)
    pub db: DbConfig,

    // User configuration (from TOML)
    pub user: UserConfig,
}

// Node configuration (filled from reth's CLI parser)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NodeConfig {
    // Chain ID (from reth)
    pub chain_id: Option<u64>,

    // Network name (mainnet, holesky, etc.)
    pub network: Option<String>,

    // Data directory
    pub datadir: Option<PathBuf>,
}

// Database configuration (from TOML)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbConfig {
    // PostgreSQL connection URL
    pub url: String,

    // Max connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

// User configuration (from TOML)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserConfig {
    // URC Registry contract address (required in config)
    pub registry_address: Address,

    // Enable backfill
    #[serde(default)]
    pub enable_backfill: bool,

    // Start block for backfill
    pub backfill_from: Option<u64>,

    // Queue capacity for event processing
    #[serde(default = "default_queue_capacity")]
    pub queue_capacity: usize,

    // Enable transaction pool monitoring
    #[serde(default)]
    pub enable_txpool_monitor: bool,
}

impl UrciConfig {
    // Load config from TOML file (db + user sections)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref())
            .wrap_err_with(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        let mut config: Self =
            toml::from_str(&contents).wrap_err("Failed to parse TOML configuration")?;

        // Node config will be filled later from reth CLI
        config.node = NodeConfig::default();

        Ok(config)
    }

    // Load from environment variable or default path
    pub fn from_env() -> Result<Self> {
        let path = std::env::var("URCI_CONFIG").unwrap_or_else(|_| "urci.toml".to_string());

        if Path::new(&path).exists() {
            Self::from_file(path)
        } else {
            eyre::bail!("Config file not found. Please provide urci.toml or set URCI_CONFIG")
        }
    }

    // Fill node config from reth's parsed arguments
    // This will be called after reth CLI is parsed
    pub fn fill_node_config(&mut self, chain_id: Option<u64>, network: Option<String>) {
        self.node.chain_id = chain_id;
        self.node.network = network;
        // Add more fields as needed when we integrate with reth
    }
}

// Default value functions (only for reasonable defaults)
fn default_max_connections() -> u32 {
    10
}
fn default_queue_capacity() -> usize {
    50_000
}

// Backwards compatibility
pub type Config = UrciConfig;
pub type GlobalConfig = UrciConfig;

pub fn load_config(path: PathBuf) -> Result<UrciConfig> {
    UrciConfig::from_file(path)
}
