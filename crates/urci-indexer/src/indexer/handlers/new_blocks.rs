//! NewBlocks handler - processes new canonical blocks

use alloy_primitives::B256;
use alloy_rpc_types_eth;
use eyre::Result;
use reth_ethereum_primitives;
use reth_primitives;
use reth_primitives_traits;
use reth_rpc_convert;
use reth_rpc_eth_api;
use reth_storage_api;
use tracing::{info, warn};
use urci_common::UrciBlockRangeUpdate;
use urci_db_adapter::{AdapterWriter, PostgresAdapter};
use urci_tracing::UrciTracer;

use crate::indexer::batch_processor::{process_block_stream, BatchProcessor};
use crate::indexer::gap_healer::{detect_gap, heal_gap, log_gap_detection};

/// Handle NewBlocks update
///
/// Steps:
/// 1. Extract consensus finality data
/// 2. Get last work_id from DB
/// 3. Check for gaps and heal if necessary
/// 4. Process incoming blocks
/// 5. Apply consensus finalization
pub async fn handle_new_blocks<EthApi>(
    range_update: UrciBlockRangeUpdate,
    adapter: &mut PostgresAdapter,
    tracer: &UrciTracer<EthApi>,
    chain_id_i64: i64,
    start_height: u64,
) -> Result<()>
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
    info!(
        "Indexer received new blocks: {:?}-{:?}",
        range_update.from, range_update.to
    );

    // Extract consensus finality data before consuming range_update
    let finalized_block = range_update.finalized_block;
    let _safe_block = range_update.safe_block; // Reserved for future metrics
    let _head_block = range_update.head_block; // Reserved for future metrics

    // TODO: Move to reader
    // CRITICAL: Get the last work_id from database to chain blocks across notifications
    // Query for latest CANONICAL block (not finalized), so we can chain new blocks to it
    let latest_canonical = sqlx::query_as::<_, (i64, Vec<u8>)>(
        "SELECT number, work_id FROM blocks WHERE canonical = true ORDER BY number DESC LIMIT 1",
    )
    .fetch_optional(adapter.pool())
    .await?;

    let mut last_work_id = latest_canonical
        .as_ref()
        .map(|(_, work_id_bytes)| B256::from_slice(work_id_bytes));
    let db_block_number = latest_canonical.as_ref().map(|(num, _)| *num as u64);

    info!(
        "DB state: latest canonical block = {:?}, last_work_id = {:?}",
        db_block_number, last_work_id
    );

    // GAP DETECTION AND HEALING
    let (gap_exists, expected_block, gap_size) = detect_gap(db_block_number, range_update.from);
    if gap_exists {
        log_gap_detection(db_block_number, expected_block, range_update.from, gap_size);

        // Heal the gap if blocks are missing
        if range_update.from > expected_block {
            last_work_id = heal_gap(
                expected_block,
                range_update.from,
                tracer,
                adapter,
                last_work_id,
                start_height,
            )
            .await?;

            info!("✓ Gap healed, now processing incoming blocks");
        }
    }

    // Process incoming blocks
    let mut processor = BatchProcessor::new(last_work_id, start_height);
    let stream = range_update.into_blocks_stream();
    process_block_stream(stream, &mut processor, adapter).await?;

    // Apply consensus finalization after all blocks are written
    if let Some(finalized) = finalized_block {
        apply_finalization(adapter, chain_id_i64, finalized).await?;
    }

    Ok(())
}

/// Apply consensus finalization to blocks
async fn apply_finalization(
    adapter: &mut PostgresAdapter,
    chain_id_i64: i64,
    finalized: alloy_eips::BlockNumHash,
) -> Result<()> {
    info!(
        "Applying consensus finalization up to block {} (hash: {})",
        finalized.number, finalized.hash
    );

    // Call existing finalize_blocks() which marks blocks as finalized AND deletes empty blocks
    match adapter
        .finalize_blocks(chain_id_i64, finalized.number as i64)
        .await
    {
        Ok(finalized_work_id) => {
            info!(
                "✓ Finalized blocks up to consensus finalized block {} (hash: {}, work_id: {})",
                finalized.number, finalized.hash, finalized_work_id
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                "Failed to finalize blocks (finalized_block={}, hash={}): {}. \
                 This is not fatal - finalization will be retried on next batch",
                finalized.number, finalized.hash, e
            );
            // Non-fatal error, continue processing
            Ok(())
        }
    }
}
