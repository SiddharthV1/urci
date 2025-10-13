//! BlsMerkleHandle struct for interacting with BLS executor thread

use eyre::Result;
use tokio::sync::mpsc;
use super::types::EvmThreadRequest;

/// Unified handle for interacting with the BLS executor thread
/// Works in both sync and async contexts
#[derive(Clone)]
pub struct BlsMerkleHandle {
    pub(super) sender: mpsc::Sender<EvmThreadRequest>,
}

impl BlsMerkleHandle {
    /// Create a handle with the given sender
    pub(crate) fn new(sender: mpsc::Sender<EvmThreadRequest>) -> Self {
        Self { sender }
    }

    /// Shutdown the executor thread gracefully
    pub async fn shutdown(&self) -> Result<()> {
        self.sender
            .send(EvmThreadRequest::Shutdown)
            .await
            .map_err(|_| eyre::eyre!("EVM executor thread already dead"))?;
        Ok(())
    }
}
