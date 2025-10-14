//! TxPool monitoring for pending transactions

use alloy_primitives::Address;
use futures::StreamExt;
use reth_transaction_pool::TransactionPool;
use tracing::warn;
use urci_tracing::UrciTracer;

/// TaskSpawner trait for spawning background tasks
pub trait TaskSpawner {
    fn spawn(&self, fut: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);
}

/// Spawn a background task to monitor the transaction pool
///
/// This monitors pending transactions destined for the registry contract
/// and traces them immediately for faster indexing.
pub fn spawn_txpool_monitor<Pool, EthApi, Executor>(
    pool: Pool,
    tracer: UrciTracer<EthApi>,
    registry_address: Address,
    executor: Executor,
) where
    Pool: TransactionPool + 'static,
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
    Executor: TaskSpawner,
{
    tracing::info!("Spawning txpool monitor");

    let mut pending_txs = pool.new_pending_pool_transactions_listener();

    executor.spawn(Box::pin(async move {
        while let Some(event) = pending_txs.next().await {
            let tx = event.transaction;
            if let Some(to) = tx.to() {
                if to == registry_address {
                    if let Err(e) = tracer.trace_pending_tx(tx).await {
                        warn!("Failed to trace pending tx: {}", e);
                    }
                }
            }
        }
    }));

    tracing::info!("✓ TxPool monitor spawned");
}
