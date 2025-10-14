//! Streaming block range updates

use alloy_eips::BlockNumHash;
use alloy_primitives::{Address, BlockNumber};
use reth_execution_types::Chain;

use super::block_update::UrciBlockUpdate;
use super::urci_tx_event_kind::UrciTxEventKind;

// Streaming version of block range update
pub struct UrciBlockRangeUpdate {
    pub from: BlockNumber,
    pub to: BlockNumber,
    // Consensus layer finalized block (from beacon chain)
    pub finalized_block: Option<BlockNumHash>,
    // Consensus layer safe block (justified checkpoint)
    pub safe_block: Option<BlockNumHash>,
    // Canonical head block
    pub head_block: BlockNumHash,
    blocks: futures::stream::BoxStream<'static, UrciBlockUpdate>,
    has_events: bool,
}

impl std::fmt::Debug for UrciBlockRangeUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UrciBlockRangeUpdate")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("finalized_block", &self.finalized_block)
            .field("safe_block", &self.safe_block)
            .field("head_block", &self.head_block)
            .field("has_events", &self.has_events)
            .finish()
    }
}

impl UrciBlockRangeUpdate {
    // Create an empty streaming update (no blocks)
    pub fn empty(from: BlockNumber, to: BlockNumber, head: BlockNumHash) -> Self {
        use futures::stream;
        use futures::StreamExt;

        Self {
            from,
            to,
            finalized_block: None,
            safe_block: None,
            head_block: head,
            blocks: stream::empty().boxed(),
            has_events: false,
        }
    }

    // Create streaming update from chain
    pub fn from_chain_streaming(
        chain: std::sync::Arc<Chain>,
        registry: Address,
        verifier: std::sync::Arc<impl crate::Verifier + 'static>,
    ) -> Self {
        use futures::StreamExt;

        let range = chain.range();
        let tip = chain.tip().num_hash();
        let blocks_stream = Self::stream_from_chain(chain, registry, verifier).boxed();

        Self {
            from: *range.start(),
            to: *range.end(),
            // Consensus data initially empty - will be enriched by ExEx
            finalized_block: None,
            safe_block: None,
            head_block: tip,
            blocks: blocks_stream,
            has_events: false, // Will be determined by consumer
        }
    }

    // Get the stream of blocks
    pub fn into_blocks_stream(self) -> futures::stream::BoxStream<'static, UrciBlockUpdate> {
        self.blocks
    }

    pub fn has_events(&self) -> bool {
        self.has_events
    }

    // Stream blocks directly from a Chain
    pub fn stream_from_chain(
        chain: std::sync::Arc<Chain>,
        registry: Address,
        verifier: std::sync::Arc<impl crate::Verifier + 'static>,
    ) -> impl futures::Stream<Item = UrciBlockUpdate> + 'static {
        use futures::{stream, StreamExt};

        let blocks_and_receipts_vec: Vec<_> = chain
            .blocks_and_receipts()
            .map(|(block, receipts)| (block.clone(), receipts.clone()))
            .collect();

        stream::iter(blocks_and_receipts_vec).then(move |(block, receipts)| {
            let verifier = verifier.clone();
            async move {
                let block_number = block.number;
                let block_hash = block.hash();
                let parent_block_hash = block.parent_hash;

                let mut block_events = Vec::new();

                for (tx_idx, ((_sender, tx), receipt)) in block
                    .transactions_with_sender()
                    .zip(receipts.iter())
                    .enumerate()
                {
                    let urc_logs: Vec<_> = receipt
                        .logs
                        .iter()
                        .enumerate()
                        .filter(|(_, log)| log.address == registry)
                        .map(|(log_idx, log)| (log.clone(), log_idx as u64))
                        .collect();

                    if !urc_logs.is_empty() {
                        match UrciTxEventKind::new(
                            &registry,
                            tx,
                            tx_idx as u64,
                            urc_logs,
                            verifier.as_ref(),
                        )
                        .await
                        {
                            Ok(tx_event_kind) => {
                                // TraceRequest will be enriched later by BatchProcessor
                                block_events.push(tx_event_kind);
                            }
                            Err(system_error) => {
                                // SystemError encountered - return block with error, no events
                                return UrciBlockUpdate {
                                    block_number,
                                    block_hash,
                                    parent_block_hash,
                                    timestamp: block.timestamp,
                                    work_id: None,
                                    parent_work_id: None,
                                    events: vec![],
                                    reorged: false,
                                    system_error: Some(system_error),
                                };
                            }
                        }
                    }
                }

                UrciBlockUpdate {
                    block_number,
                    block_hash,
                    parent_block_hash,
                    timestamp: block.timestamp,
                    work_id: None,
                    parent_work_id: None,
                    events: block_events,
                    reorged: false,
                    system_error: None,
                }
            }
        })
    }

    pub async fn from_chain(
        chain: &std::sync::Arc<Chain>,
        registry: Address,
        verifier: std::sync::Arc<impl crate::Verifier + 'static>,
    ) -> eyre::Result<Self> {
        Ok(Self::from_chain_streaming(
            chain.clone(),
            registry,
            verifier,
        ))
    }
}
