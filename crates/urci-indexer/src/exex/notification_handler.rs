//! ExEx notification processing loop

use alloy_primitives::Address;
use eyre::Result;
use futures::TryStreamExt;
use reth_exex::{ExExContext, ExExEvent, ExExNotification};
use reth_node_api::{FullNodeComponents, NodeTypes};
use reth_primitives::EthPrimitives;
use std::sync::Arc;
use tracing::{debug, error, info, warn, instrument};

use super::consensus::query_consensus_data;

/// Main ExEx loop - streams blocks to indexer
#[instrument(skip(ctx, indexer, verifier), fields(registry = %registry))]
pub async fn run_exex<Node, V>(
    mut ctx: ExExContext<Node>,
    registry: Address,
    indexer: impl urci_common::Indexer + 'static,
    verifier: Arc<V>,
) -> Result<()>
where
    Node: FullNodeComponents<Types: NodeTypes<Primitives = EthPrimitives>>,
    V: urci_common::Verifier + 'static,
{
    use urci_common::{UrciBlockRangeUpdate, UrciBlockRangeUpdateKind};

    info!("ExEx loop started, waiting for notifications");

    // Main notification processing loop
    while let Some(notification) = ctx.notifications.try_next().await.map_err(|e| {
        error!(error = %e, "Failed to receive ExEx notification");
        e
    })? {
        debug!("Received notification: {:?}", notification);

        let result = match &notification {
            ExExNotification::ChainCommitted { new } => {
                let range = new.range();
                let tip = new.tip().num_hash();

                // Query consensus finality data
                let (finalized, safe, head) = query_consensus_data(&ctx, tip);

                info!(
                    from_block = range.start(),
                    to_block = range.end() - 1,
                    finalized = ?finalized.map(|f| f.number),
                    safe = ?safe.map(|s| s.number),
                    head = head.number,
                    "Processing committed chain with consensus data"
                );

                // Create streaming update from the chain
                let mut range_update = UrciBlockRangeUpdate::from_chain_streaming(
                    new.clone(),
                    registry,
                    verifier.clone(),
                );

                // Enrich with consensus finality data
                range_update.finalized_block = finalized;
                range_update.safe_block = safe;
                range_update.head_block = head;

                // Send to indexer
                indexer.index(UrciBlockRangeUpdateKind::NewBlocks(range_update)).await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            from_block = range.start(),
                            to_block = range.end() - 1,
                            "Failed to index committed chain"
                        );
                        e
                    })
            },
            ExExNotification::ChainReorged { old, new } => {
                let old_range = old.range();
                let new_range = new.range();
                let new_tip = new.tip().num_hash();

                // Query consensus finality data
                let (finalized, safe, head) = query_consensus_data(&ctx, new_tip);

                warn!(
                    old_from = old_range.start(),
                    old_to = old_range.end() - 1,
                    old_tip_hash = ?old.tip().hash(),
                    new_from = new_range.start(),
                    new_to = new_range.end() - 1,
                    new_tip_hash = ?new.tip().hash(),
                    finalized = ?finalized.map(|f| f.number),
                    safe = ?safe.map(|s| s.number),
                    head = head.number,
                    "Processing reorg with consensus data"
                );

                // Create reorg update for indexer
                let from = (old.tip().number, old.tip().hash());
                let to = (new.tip().number, new.tip().hash());
                let mut new_chain = UrciBlockRangeUpdate::from_chain_streaming(
                    new.clone(),
                    registry,
                    verifier.clone(),
                );

                // Enrich with consensus finality data
                new_chain.finalized_block = finalized;
                new_chain.safe_block = safe;
                new_chain.head_block = head;

                indexer.index(UrciBlockRangeUpdateKind::Reorg { from, to, new_chain }).await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            old_tip = old.tip().number,
                            new_tip = new.tip().number,
                            "Failed to process reorg"
                        );
                        e
                    })
            },
            ExExNotification::ChainReverted { old } => {
                let range = old.range();
                warn!(
                    from_block = range.start(),
                    to_block = range.end() - 1,
                    tip_hash = ?old.tip().hash(),
                    "Processing revert"
                );

                // Send revert to indexer
                let from = (old.first().number, old.first().hash());
                let to = (old.tip().number, old.tip().hash());

                indexer.index(UrciBlockRangeUpdateKind::Revert { from, to }).await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            from_block = from.0,
                            to_block = to.0,
                            "Failed to process revert"
                        );
                        e
                    })
            },
        };

        // Propagate errors after logging
        result?;

        // Send confirmation after processing
        if let Some(committed_chain) = notification.committed_chain() {
            let tip = committed_chain.tip().num_hash();
            debug!(block_number = tip.number, block_hash = ?tip.hash, "Sending FinishedHeight event");

            ctx.events.send(ExExEvent::FinishedHeight(tip)).map_err(|e| {
                error!(error = %e, block_number = tip.number, "Failed to send FinishedHeight event");
                eyre::eyre!("Failed to send FinishedHeight: {}", e)
            })?;
        }
    }

    warn!("ExEx notification stream closed - shutting down");
    Ok(())
}
