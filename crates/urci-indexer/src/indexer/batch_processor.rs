//! Shared batch processing logic
//!
//! This module contains the BatchProcessor which handles the common pattern of:
//! - Trace enrichment for indirect calls (TraceRequest → UrciEvent)
//! - Accumulating blocks into batches
//! - Checking for SystemErrors
//! - Setting parent_work_id chains
//! - Writing batches when full (100 blocks)
//! - Flushing remaining blocks

use alloy_primitives::B256;
use eyre::Result;
use futures::StreamExt;
use tracing::{error, info};
use urci_common::{UrciBatchedBlockRangeUpdate, UrciTxEventKind};
use urci_db_adapter::{AdapterWriter, PostgresAdapter};
use urci_tracing::UrciTracer;

/// Batch size for writing blocks to database
const BATCH_SIZE: usize = 100;

/// Processes blocks in batches with trace enrichment and SystemError handling
pub struct BatchProcessor<EthApi> {
    batch: UrciBatchedBlockRangeUpdate,
    last_work_id: Option<B256>,
    start_height: u64,
    tracer: UrciTracer<EthApi>,
}

impl<EthApi> BatchProcessor<EthApi>
where
    EthApi: reth_rpc_eth_api::helpers::FullEthApi
        + reth_rpc_eth_api::helpers::LoadBlock
        + Clone
        + Send
        + Sync
        + 'static,
    <EthApi::NetworkTypes as reth_rpc_convert::RpcTypes>::TransactionRequest:
        From<alloy_rpc_types_eth::TransactionRequest>,
    <EthApi as reth_rpc_eth_api::node::RpcNodeCore>::Provider:
        reth_storage_api::BlockReader + reth_storage_api::ReceiptProvider,
    <EthApi as reth_rpc_eth_api::node::RpcNodeCore>::Primitives:
        reth_primitives_traits::node::NodePrimitives<
            Block = reth_primitives::Block,
            Receipt = reth_ethereum_primitives::EthereumReceipt,
        >,
{
    /// Create a new batch processor with tracer for enrichment
    ///
    /// - `last_work_id`: The work_id to chain to (from last write)
    /// - `start_height`: The chain root height (blocks at this height don't get parent_work_id)
    /// - `tracer`: UrciTracer for enriching TraceRequests
    pub fn new(last_work_id: Option<B256>, start_height: u64, tracer: UrciTracer<EthApi>) -> Self {
        Self {
            batch: UrciBatchedBlockRangeUpdate::new(),
            last_work_id,
            start_height,
            tracer,
        }
    }

    /// Process a single block with trace enrichment
    ///
    /// Steps:
    /// 1. Check events for TraceRequests and enrich with tracer
    /// 2. Check for SystemErrors
    /// 3. Set parent_work_id
    /// 4. Add to batch
    /// 5. Auto-write when batch reaches BATCH_SIZE
    pub async fn process_block(
        &mut self,
        mut block: urci_common::UrciBlockUpdate,
        adapter: &mut PostgresAdapter,
    ) -> Result<()> {
        // STEP 1: TRACE ENRICHMENT for indirect calls
        for event_kind in &mut block.events {
            match event_kind {
                UrciTxEventKind::UrciEvent(_) => {
                    // Already enriched - direct call with single log
                }
                UrciTxEventKind::TraceRequest(tx_event) => {
                    info!(
                        "Block {} tx {:?} needs trace enrichment (indirect call or multiple logs)",
                        block.block_number, tx_event.transaction_hash
                    );

                    // Call tracer to get full call graph
                    match self.tracer.trace_and_enrich(tx_event.clone()).await {
                        Ok(enriched) => {
                            info!(
                                "✓ Enriched tx {:?}: {} call edges, {} events",
                                enriched.transaction_hash,
                                enriched.trace.len(),
                                enriched.urc_events.len()
                            );
                            // Replace TraceRequest with enriched UrciEvent
                            *event_kind = UrciTxEventKind::UrciEvent(enriched);
                        }
                        Err(e) => {
                            error!(
                                "Failed to enrich tx {:?} in block {}: {}",
                                tx_event.transaction_hash, block.block_number, e
                            );
                            error!("Trace enrichment FAILED - flipping indexer to FAILED STATE");
                            // Flip to failed state - all future blocks go to failed_block_writes
                            adapter
                                .failed_state
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                            // Keep as TraceRequest so db_adapter writes to error table
                        }
                    }
                }
            }
        }

        // STEP 2: Check for SystemError - if present, flip to failed state
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

        // STEP 3: Set parent_work_id from the last write (but NOT for root block at start_height)
        if let Some(prev_work_id) = self.last_work_id {
            if block.block_number > self.start_height {
                block.parent_work_id = Some(prev_work_id);
            }
        }

        // STEP 4: Add to batch
        self.batch.add_block(block);

        // STEP 5: Write when batch is full
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

/// Process a stream of blocks with trace enrichment using the BatchProcessor
///
/// Helper function that handles the common pattern of streaming blocks and batching them
pub async fn process_block_stream<S, EthApi>(
    mut stream: S,
    processor: &mut BatchProcessor<EthApi>,
    adapter: &mut PostgresAdapter,
) -> Result<()>
where
    S: futures::Stream<Item = urci_common::UrciBlockUpdate> + Unpin,
    EthApi: reth_rpc_eth_api::helpers::FullEthApi
        + reth_rpc_eth_api::helpers::LoadBlock
        + Clone
        + Send
        + Sync
        + 'static,
    <EthApi::NetworkTypes as reth_rpc_convert::RpcTypes>::TransactionRequest:
        From<alloy_rpc_types_eth::TransactionRequest>,
    <EthApi as reth_rpc_eth_api::node::RpcNodeCore>::Provider:
        reth_storage_api::BlockReader + reth_storage_api::ReceiptProvider,
    <EthApi as reth_rpc_eth_api::node::RpcNodeCore>::Primitives:
        reth_primitives_traits::node::NodePrimitives<
            Block = reth_primitives::Block,
            Receipt = reth_ethereum_primitives::EthereumReceipt,
        >,
{
    while let Some(block) = stream.next().await {
        processor.process_block(block, adapter).await?;
    }

    // Flush remaining blocks
    processor.flush(adapter).await?;

    Ok(())
}
