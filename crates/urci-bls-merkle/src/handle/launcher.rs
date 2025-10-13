//! Launcher for BLS executor thread

use eyre::Result;
use tokio::sync::mpsc;
use std::time::Duration;
use crate::cached_executor::CachedBlsExecutor;
use crate::evm::EvmBlsMerkleExecutor;
use super::client::BlsMerkleHandle;
use super::types::EvmThreadRequest;
use super::worker::run_worker;

/// Launch the BLS executor in a separate thread and return a unified handle
pub fn launch_bls_executor() -> Result<BlsMerkleHandle> {
    let (tx, rx) = mpsc::channel::<EvmThreadRequest>(100);

    // Spawn the executor thread
    std::thread::spawn(move || {
        // Create EVM executor (this spawns its own internal thread)
        let evm_executor = match EvmBlsMerkleExecutor::new() {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("Failed to create EVM executor: {}", e);
                return;
            }
        };

        // Wrap with caching
        let cached = CachedBlsExecutor::new(evm_executor)
            .with_config(
                Duration::from_secs(600), // 10 minute TTL
                50_000,                    // Up to 50k entries per cache
                true,                      // Cache merkle proofs
            );

        // Run the worker loop
        run_worker(rx, cached);
    });

    Ok(BlsMerkleHandle::new(tx))
}
