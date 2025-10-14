//! Reorg handler - processes chain reorganizations

use alloy_primitives::B256;
use eyre::Result;
use tracing::{info, warn};
use urci_common::UrciBlockRangeUpdate;
use urci_db_adapter::{AdapterWriter, PostgresAdapter};

use crate::indexer::batch_processor::{process_block_stream, BatchProcessor};

/// Handle Reorg update
///
/// Steps:
/// 1. Mark old chain as non-canonical
/// 2. Process new canonical chain
/// 3. Apply consensus finalization
pub async fn handle_reorg(
    from: (u64, B256),
    to: (u64, B256),
    new_chain: UrciBlockRangeUpdate,
    adapter: &mut PostgresAdapter,
    chain_id_i64: i64,
) -> Result<()> {
    info!(
        "Indexer handling reorg: marking blocks {} to {} as non-canonical",
        from.0, to.0
    );

    // Extract consensus finality data before consuming new_chain
    let finalized_block = new_chain.finalized_block;
    let _safe_block = new_chain.safe_block; // Reserved for future metrics
    let _head_block = new_chain.head_block; // Reserved for future metrics

    // Mark the exact range from..=to as non-canonical
    let parent_work_id = adapter.handle_reorg(from.0, to.0).await?;

    // Write new canonical chain - use parent_work_id from reorg as starting point
    info!(
        "Processing new canonical chain blocks {}-{}",
        new_chain.from, new_chain.to
    );
    info!(
        "Starting reorg chain with parent_work_id from reorg: {:?}",
        parent_work_id
    );

    // Process new canonical blocks
    // Note: For reorg blocks, we don't skip parent_work_id even if at start_height
    // because these are replacing an existing chain
    let mut processor = BatchProcessor::new(Some(parent_work_id), 0);
    let stream = new_chain.into_blocks_stream();
    process_block_stream(stream, &mut processor, adapter).await?;

    // Apply consensus finalization after all blocks are written
    if let Some(finalized) = finalized_block {
        apply_finalization_after_reorg(adapter, chain_id_i64, finalized).await?;
    }

    Ok(())
}

/// Apply consensus finalization after a reorg
async fn apply_finalization_after_reorg(
    adapter: &mut PostgresAdapter,
    chain_id_i64: i64,
    finalized: alloy_eips::BlockNumHash,
) -> Result<()> {
    info!(
        "Applying consensus finalization after reorg up to block {} (hash: {})",
        finalized.number, finalized.hash
    );

    // Call existing finalize_blocks() which marks blocks as finalized AND deletes empty blocks
    match adapter
        .finalize_blocks(chain_id_i64, finalized.number as i64)
        .await
    {
        Ok(finalized_work_id) => {
            info!(
                "✓ Finalized blocks after reorg up to block {} (hash: {}, work_id: {})",
                finalized.number, finalized.hash, finalized_work_id
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                "Failed to finalize blocks after reorg (finalized_block={}, hash={}): {}. \
                 This is not fatal - finalization will be retried on next batch",
                finalized.number, finalized.hash, e
            );
            // Non-fatal error, continue processing
            Ok(())
        }
    }
}
