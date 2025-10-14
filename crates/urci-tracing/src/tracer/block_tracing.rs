//! Block transaction tracing (full call graph + BLS validation)

use alloy_primitives::B256;
use eyre::Result;
use reth_rpc_eth_api::helpers::{FullEthApi, LoadBlock};
use reth_storage_api::{BlockReader, ReceiptProvider};
use tracing::{debug, info};
use urci_common::{CallTrace, UrciTxEvent};

use crate::inspectors::UrciInspector;
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
    /// Trace and enrich a UrciTxEvent (TraceRequest -> UrcEvent)
    ///
    /// Uses specialized inspector for block transaction tracing
    pub async fn trace_and_enrich(&self, trace_request: UrciTxEvent) -> Result<UrciTxEvent> {
        let tx_hash = trace_request.transaction_hash;
        debug!("Tracing block tx for enrichment: {:?}", tx_hash);

        // Create inspector for block tracing (both call graph and BLS validation)
        let inspector = UrciInspector::for_tracing(
            self.config.registry_address,
            tx_hash,
            0,          // block_number from trace_request
            B256::ZERO, // block_hash from trace_request
            trace_request.transaction_index as u32,
        )
        .with_bls_handle((*self.bls_handle).clone());

        // Clone data needed for closure
        let _registry_address = self.config.registry_address;
        let _bls_handle = self.bls_handle.clone();
        let _tx_sender = trace_request.tx_sender_address;

        // Use spawn_trace_transaction_in_block_with_inspector
        let trace_result = self
            .eth_api
            .spawn_trace_transaction_in_block_with_inspector(
                tx_hash,
                inspector,
                move |_tx_info, inspector, result, _db| {
                    // Extract data from inspector
                    let validation_results = inspector.get_validation_results();
                    let log_to_call_map = inspector.get_log_to_call_map().to_vec(); // Clone before consuming
                    let edges = inspector.get_edges(); // Consumes inspector

                    // Get logs from result - these are the actual EVM logs emitted
                    let logs = result.result.logs();

                    info!(
                        "Traced tx {:?}: {} edges, {} logs, {} validations, {} log mappings",
                        tx_hash,
                        edges.len(),
                        logs.len(),
                        validation_results.len(),
                        log_to_call_map.len()
                    );

                    // Now we have the mapping from log index -> (call_depth, call_index_at_depth)
                    // This was captured in the inspector's log() callback when each log was emitted

                    // Build enriched event with trace
                    let mut enriched = trace_request.clone();
                    enriched.trace = edges.into_iter().map(|_e| CallTrace).collect();

                    // Note: urc_events will need to be populated asynchronously outside this closure
                    // because UrcEventKind::from_log_and_input is async

                    info!(
                        "Traced tx {:?}: {} call edges captured",
                        tx_hash,
                        enriched.trace.len()
                    );

                    Ok(enriched)
                },
            )
            .await?;

        trace_result.ok_or_else(|| eyre::eyre!("Transaction not found: {:?}", tx_hash))
    }
}
