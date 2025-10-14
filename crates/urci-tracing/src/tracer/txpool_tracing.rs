//! Txpool transaction tracing (lightweight BLS validation only)

use eyre::Result;
use reth_rpc_eth_api::helpers::{FullEthApi, LoadBlock};
use reth_storage_api::{BlockReader, ReceiptProvider};
use tracing::{debug, error, info, instrument, warn};
use urci_common::CallTrace;

use crate::inspectors::UrcTxPoolInspector;
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
    /// Trace a pending transaction from txpool
    ///
    /// Uses UrcTxPoolInspector (lightweight, BLS validation only)
    #[instrument(skip(self, tx), fields(tx_hash = ?tx.transaction.hash()))]
    pub async fn trace_pending_tx<T>(&self, tx: std::sync::Arc<reth_transaction_pool::ValidPoolTransaction<T>>) -> Result<Vec<CallTrace>>
    where
        T: reth_transaction_pool::PoolTransaction,
    {
        let tx_hash = *tx.transaction.hash();
        debug!("Tracing pending tx from txpool");

        // Create UrcTxPoolInspector for lightweight txpool validation
        // This inspector only validates BLS signatures, doesn't capture call graphs
        let inspector = UrcTxPoolInspector::new(self.config.registry_address, tx_hash)
            .with_bls_handle((*self.bls_handle).clone());

        // Trace the pending transaction using the txpool inspector
        let trace_result = self
            .eth_api
            .spawn_trace_transaction_in_block_with_inspector(
                tx_hash,
                inspector,
                move |_tx_info, inspector, _result, _db| {
                    // Get validation results from inspector
                    let validation_results = inspector.get_validation_results();
                    let call_succeeded = inspector.registry_call_succeeded();

                    info!(
                        validations_count = validation_results.len(),
                        call_succeeded = call_succeeded,
                        "Traced pending tx"
                    );

                    // If call didn't succeed, don't cache validation (failed tx won't be included)
                    if !call_succeeded {
                        debug!("Registry call failed in pending tx, skipping");
                        return Ok(vec![]);
                    }

                    // For valid registrations, cache them in the BLS executor
                    // so when the tx is included in a block, we don't re-validate
                    // TODO: Cache validation results in BLS executor

                    Ok(vec![])
                },
            )
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to trace pending transaction");
                e
            })?;

        trace_result.ok_or_else(|| {
            warn!("Pending transaction not found in mempool");
            eyre::eyre!("Pending transaction not found: {:?}", tx_hash)
        })
    }
}
