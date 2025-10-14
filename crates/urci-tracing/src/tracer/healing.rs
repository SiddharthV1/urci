//! Historical block healing - fetch and process missing blocks

use eyre::Result;
use reth_execution_types::ExecutionOutcome;
use reth_provider::Chain;
use reth_rpc_eth_api::helpers::{FullEthApi, LoadBlock};
use reth_storage_api::{BlockReader, ReceiptProvider};
use revm::database::BundleState;
use std::sync::Arc;
use tracing::info;
use urci_common::UrciBlockRangeUpdate;

use super::urci_tracer::UrciTracer;

impl<EthApi> UrciTracer<EthApi>
where
    EthApi: FullEthApi + LoadBlock,
    <EthApi::NetworkTypes as reth_rpc_convert::RpcTypes>::TransactionRequest:
        From<alloy_rpc_types_eth::TransactionRequest>,
    <EthApi as reth_rpc_eth_api::node::RpcNodeCore>::Provider: BlockReader + ReceiptProvider,
    <EthApi as reth_rpc_eth_api::node::RpcNodeCore>::Primitives: reth_primitives_traits::node::NodePrimitives<
        Block = reth_primitives::Block,
        Receipt = reth_ethereum_primitives::EthereumReceipt,
    >,
{
    /// Heal missing blocks - fetch historical blocks and process them
    ///
    /// Uses Provider to fetch blocks with senders and receipts, constructs a Chain,
    /// then processes it using the same infrastructure as the ExEx.
    ///
    /// Returns a streaming block range update for scalable healing of large gaps.
    pub async fn heal_missing_blocks(&self, from: u64, to: u64) -> Result<UrciBlockRangeUpdate> {
        info!("Healing missing blocks {} to {}", from, to);

        // Access provider through FullEthApi (which extends RpcNodeCore)
        let provider = self.eth_api.provider();

        // Fetch blocks with senders (RecoveredBlock has sender info)
        let blocks = provider
            .block_with_senders_range(from..=to)
            .map_err(|e| eyre::eyre!("Failed to fetch blocks {}-{}: {}", from, to, e))?;

        if blocks.is_empty() {
            return Err(eyre::eyre!("Provider returned no blocks for range {}-{}", from, to));
        }

        // Fetch receipts for the same range
        let receipts = provider
            .receipts_by_block_range(from..=to)
            .map_err(|e| eyre::eyre!("Failed to fetch receipts {}-{}: {}", from, to, e))?;

        if receipts.len() != blocks.len() {
            return Err(eyre::eyre!(
                "Mismatch: {} blocks but {} receipt sets",
                blocks.len(),
                receipts.len()
            ));
        }

        // Construct ExecutionOutcome (we only need receipts for URC processing)
        type ProviderReceipt<E> = <<E as reth_rpc_eth_api::node::RpcNodeCore>::Provider as ReceiptProvider>::Receipt;
        type ProviderPrimitives<E> = <E as reth_rpc_eth_api::node::RpcNodeCore>::Primitives;

        let execution_outcome = ExecutionOutcome::<ProviderReceipt<EthApi>>::new(
            BundleState::default(), // Empty state changes (not needed for URC events)
            receipts,               // Receipts for each block
            from,                   // First block number
            vec![],                 // Empty requests
        );

        // Construct Chain from blocks and execution outcome
        let chain = Chain::<ProviderPrimitives<EthApi>>::new(
            blocks,
            execution_outcome,
            None, // No trie updates needed
        );

        info!(
            "Constructed chain for healing: blocks {}-{} ({} blocks)",
            from,
            to,
            chain.len()
        );

        // Convert to Arc<Chain<EthPrimitives>> which is what from_chain_streaming expects
        // This is safe because we constrained Primitives to have the same Block/Receipt types
        let chain_arc: Arc<Chain> = unsafe {
            // SAFETY: The trait bounds ensure that ProviderPrimitives<EthApi> has the same
            // Block and Receipt types as EthPrimitives (Ethereum's default primitives).
            // The Chain struct is repr(transparent) over its fields, so this transmute is safe.
            std::mem::transmute(Arc::new(chain))
        };

        // Return streaming update (not Vec!) for scalable healing
        let range_update = UrciBlockRangeUpdate::from_chain_streaming(
            chain_arc,
            self.config.registry_address,
            self.bls_handle.clone(),
        );

        info!("Healing stream prepared for blocks {}-{}", from, to);

        Ok(range_update)
    }
}
