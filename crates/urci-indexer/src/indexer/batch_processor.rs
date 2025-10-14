//! Shared batch processing logic
//!
//! This module contains the BatchProcessor which handles the common pattern of:
//! - Accumulating blocks into batches
//! - Checking for SystemErrors
//! - Setting parent_work_id chains
//! - Writing batches when full (100 blocks)
//! - Flushing remaining blocks

use alloy_primitives::B256;
use eyre::Result;
use futures::StreamExt;
use tracing::{error, info};
use urci_common::UrciBatchedBlockRangeUpdate;
use urci_db_adapter::{AdapterWriter, PostgresAdapter};

/// Batch size for writing blocks to database
const BATCH_SIZE: usize = 100;

/// Processes blocks in batches with SystemError handling
pub struct BatchProcessor {
    batch: UrciBatchedBlockRangeUpdate,
    last_work_id: Option<B256>,
    start_height: u64,
}

impl BatchProcessor {
    /// Create a new batch processor
    ///
    /// - `last_work_id`: The work_id to chain to (from last write)
    /// - `start_height`: The chain root height (blocks at this height don't get parent_work_id)
    pub fn new(last_work_id: Option<B256>, start_height: u64) -> Self {
        Self {
            batch: UrciBatchedBlockRangeUpdate::new(),
            last_work_id,
            start_height,
        }
    }

    /// Process a single block, handling SystemError and parent_work_id
    ///
    /// Automatically writes batch if it reaches BATCH_SIZE
    pub async fn process_block(
        &mut self,
        mut block: urci_common::UrciBlockUpdate,
        adapter: &mut PostgresAdapter,
    ) -> Result<()> {
        // CRITICAL: Check for SystemError - if present, flip to failed state
        if let Some(ref system_error) = block.system_error {
            error!(
                "SYSTEM ERROR at block {}: {:?}",
                block.block_number, system_error
            );
            error!(
                "Flipping indexer to FAILED STATE - all future writes go to failed_block_writes"
            );
            adapter
                .failed_state
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Set parent_work_id from the last write (but NOT for root block at start_height)
        if let Some(prev_work_id) = self.last_work_id {
            if block.block_number > self.start_height {
                block.parent_work_id = Some(prev_work_id);
            }
        }

        self.batch.add_block(block);

        // Write when batch is full
        if self.batch.blocks.len() >= BATCH_SIZE {
            self.flush(adapter).await?;
        }

        Ok(())
    }

    /// Flush the current batch to database
    ///
    /// Returns the work_id of the last written block
    pub async fn flush(&mut self, adapter: &mut PostgresAdapter) -> Result<Option<B256>> {
        if self.batch.blocks.is_empty() {
            return Ok(self.last_work_id);
        }

        let batch_size = self.batch.blocks.len();
        info!("Writing batch of {} blocks to DB", batch_size);

        let work_id = adapter
            .write_blocks(std::mem::replace(
                &mut self.batch,
                UrciBatchedBlockRangeUpdate::new(),
            ))
            .await
            .map_err(|e| {
                error!("Failed to write batch of {} blocks: {:?}", batch_size, e);
                e
            })?;

        self.last_work_id = Some(work_id);
        Ok(Some(work_id))
    }

    /// Get the current last_work_id
    pub fn last_work_id(&self) -> Option<B256> {
        self.last_work_id
    }
}

/// Process a stream of blocks using the BatchProcessor
///
/// Helper function that handles the common pattern of streaming blocks and batching them
pub async fn process_block_stream<S>(
    mut stream: S,
    processor: &mut BatchProcessor,
    adapter: &mut PostgresAdapter,
) -> Result<()>
where
    S: futures::Stream<Item = urci_common::UrciBlockUpdate> + Unpin,
{
    while let Some(block) = stream.next().await {
        processor.process_block(block, adapter).await?;
    }

    // Flush remaining blocks
    processor.flush(adapter).await?;

    Ok(())
}
