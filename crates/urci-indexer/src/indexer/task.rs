//! Main indexer task
//!
//! This is the main entry point for the indexer task that:
//! - Receives UrciBlockRangeUpdateKind messages from the ExEx
//! - Dispatches to appropriate handlers based on update type
//! - Manages the overall indexing flow

use eyre::Result;
use tracing::info;
use urci_common::{UrciBlockRangeUpdateKind, UrciConfig};
use urci_db_adapter::{traits::AdapterReader, PostgresAdapter};
use urci_tracing::UrciTracer;

use super::handlers::{handle_new_blocks, handle_reorg, handle_revert};

/// Main indexer task - receives block updates and writes to DB
///
/// This is spawned as a background task and processes updates from the ExEx
pub async fn run_indexer<EthApi>(
    mut rx: tokio::sync::mpsc::Receiver<UrciBlockRangeUpdateKind>,
    mut adapter: PostgresAdapter,
    _config: UrciConfig,
    tracer: UrciTracer<EthApi>,
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
    info!("Indexer task started");

    // Determine start_height: use finalized block if exists, otherwise use chain_root
    let chain_id = adapter.chain_id();
    let chain_id_i64 = chain_id as i64;

    let start_height = if let Some((finalized_height, _, _)) =
        adapter.get_latest_finalized_block().await?
    {
        info!("Starting from latest finalized block: {}", finalized_height);
        finalized_height
    } else if let Some((root_height, _)) = adapter.get_chain_root(chain_id).await? {
        info!(
            "No finalized blocks, starting from chain_root: {}",
            root_height
        );
        root_height
    } else {
        return Err(eyre::eyre!("No finalized blocks and no chain_root found"));
    };

    info!("Start height = {}", start_height);

    // Main update processing loop
    while let Some(update) = rx.recv().await {
        let result = match update {
            UrciBlockRangeUpdateKind::NewBlocks(range_update) => {
                handle_new_blocks(
                    range_update,
                    &mut adapter,
                    &tracer,
                    chain_id_i64,
                    start_height,
                )
                .await
            }
            UrciBlockRangeUpdateKind::Reorg { from, to, new_chain } => {
                handle_reorg(from, to, new_chain, &mut adapter, &tracer, chain_id_i64).await
            }
            UrciBlockRangeUpdateKind::Revert { from, to } => {
                handle_revert(from, to, &mut adapter).await
            }
        };

        // Propagate errors
        result?;
    }

    info!("Indexer task shutting down");
    Ok(())
}
