//! Gap detection and healing logic

use alloy_primitives::B256;
use eyre::Result;
use tracing::{error, info, warn};
use urci_db_adapter::PostgresAdapter;
use urci_tracing::UrciTracer;

use super::batch_processor::BatchProcessor;

/// Detect if there's a gap between DB state and incoming blocks
///
/// Returns (gap_exists, expected_block, gap_size)
pub fn detect_gap(
    db_block_number: Option<u64>,
    incoming_block: u64,
) -> (bool, u64, u64) {
    let expected_block = db_block_number.map(|n| n + 1).unwrap_or(0);
    let gap_exists = incoming_block != expected_block;
    let gap_size = incoming_block.saturating_sub(expected_block);

    (gap_exists, expected_block, gap_size)
}

/// Heal a gap by fetching and processing missing blocks
///
/// Returns the final work_id after healing, or None if healing failed
pub async fn heal_gap<EthApi>(
    expected_block: u64,
    incoming_block: u64,
    tracer: &UrciTracer<EthApi>,
    adapter: &mut PostgresAdapter,
    last_work_id: Option<B256>,
    start_height: u64,
) -> Result<Option<B256>>
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
        "Healing gap: fetching blocks {}-{}",
        expected_block,
        incoming_block - 1
    );

    match tracer.heal_missing_blocks(expected_block, incoming_block - 1).await {
        Ok(healed_range) => {
            info!(
                "Successfully healed gap, processing healed blocks {}-{}",
                healed_range.from, healed_range.to
            );

            // Process healed blocks using BatchProcessor with trace enrichment
            let mut processor = BatchProcessor::new(last_work_id, start_height, tracer.clone());
            let healed_stream = healed_range.into_blocks_stream();

            super::batch_processor::process_block_stream(healed_stream, &mut processor, adapter)
                .await?;

            info!("✓ Gap healed successfully");
            Ok(processor.last_work_id())
        }
        Err(e) => {
            error!("Failed to heal gap: {}. Flipping to FAILED STATE", e);
            adapter
                .failed_state
                .store(true, std::sync::atomic::Ordering::Relaxed);
            // Return last_work_id unchanged so caller can continue
            Ok(last_work_id)
        }
    }
}

/// Log gap detection information
pub fn log_gap_detection(
    db_block_number: Option<u64>,
    expected_block: u64,
    incoming_block: u64,
    gap_size: u64,
) {
    warn!(
        "Gap detected! DB has block {}, expected next block {}, but received block {}. Gap size: {} blocks",
        db_block_number.unwrap_or(0),
        expected_block,
        incoming_block,
        gap_size
    );
}
