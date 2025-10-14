//! URCI Indexer - Main entry point
//!
//! Launches a reth node with URC indexing capabilities.
//! Configuration comes from:
//! - Reth CLI for node settings
//! - TOML config file for indexer settings
//!
//! Initialization flow (from indexer.md):
//! 1. Initialize DB adapter (connect, check tables, get config, create session)
//! 2. Create BLS/Merkle EVM Executor
//! 3. Launch Node with ExEx (pass start block from DB)
//! 4. Initialize Tracer
//! 5. Initialize TxPool Monitor
//! 6. Start indexer task

mod exex;
mod indexer;
mod node;

use eyre::Result;
use tracing::info;
use urci_common::UrciConfig;

fn main() -> Result<()> {
    // Set default logging if not configured
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,urci=debug");
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting URCI Indexer");

    // Load indexer configuration from TOML file
    let config = UrciConfig::from_env()?;
    info!("Loaded configuration from TOML");

    // Build and launch the node
    node::build_and_launch(config)
}
