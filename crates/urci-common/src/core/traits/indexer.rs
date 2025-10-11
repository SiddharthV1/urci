//! Indexer trait for processing blockchain updates

use super::super::block_update::UrciBlockRangeUpdateKind;
use async_trait::async_trait;
use eyre::Result;
use tokio::sync::mpsc;
use tracing::{debug, error, instrument};

// Trait for indexing blockchain updates
//
// This trait defines the interface that ExEx uses to communicate with the indexer.
// The ExEx will await the `index` method to complete before continuing processing.
#[async_trait]
pub trait Indexer: Send + Sync {
    // Process a blockchain update (new blocks, reorg, or revert)
    //
    // The ExEx will call this method and await its completion before processing
    // the next notification, providing natural backpressure.
    async fn index(&self, update: UrciBlockRangeUpdateKind) -> Result<()>;
}

// Handle for communicating with the indexer running in another context
//
// This handle implements the Indexer trait and sends updates to the actual
// indexer via a channel. The ExEx holds this handle and uses it to send
// blockchain updates, while the actual indexer runs in a separate task.
//
// # Example
// ```ignore
// // Create channel for communication
// let (handle, rx) = IndexerHandle::channel(1000);
//
// // Spawn the actual indexer in a separate task
// tokio::spawn(async move {
//     let indexer = MyIndexer::new();
//     while let Some(update) = rx.recv().await {
//         indexer.index(update).await.unwrap();
//     }
// });
//
// // Pass handle to ExEx
// let exex = MyExEx::new(handle);
// ```
#[derive(Clone)]
pub struct IndexerHandle {
    tx: mpsc::Sender<UrciBlockRangeUpdateKind>,
}

impl IndexerHandle {
    // Create a new indexer handle with the given sender
    pub fn new(tx: mpsc::Sender<UrciBlockRangeUpdateKind>) -> Self {
        Self { tx }
    }

    // Create a new indexer handle and receiver pair
    pub fn channel(buffer: usize) -> (Self, mpsc::Receiver<UrciBlockRangeUpdateKind>) {
        let (tx, rx) = mpsc::channel(buffer);
        (Self::new(tx), rx)
    }
}

#[async_trait]
impl Indexer for IndexerHandle {
    #[instrument(skip(self, update), fields(update_type = ?update))]
    async fn index(&self, update: UrciBlockRangeUpdateKind) -> Result<()> {
        debug!("Sending update to indexer task");

        self.tx
            .send(update)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send update to indexer - channel closed or full");
                eyre::eyre!("Failed to send update to indexer: {}", e)
            })?;

        debug!("Update sent successfully");
        Ok(())
    }
}
