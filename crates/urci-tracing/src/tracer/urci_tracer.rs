//! Main UrciTracer struct and constructors

use alloy_primitives::Address;
use reth_rpc_eth_api::helpers::{FullEthApi, LoadBlock};
use reth_storage_api::{BlockReader, ReceiptProvider};
use std::sync::Arc;
use tokio::sync::RwLock;
use urci_bls_merkle::BlsMerkleHandle;
use urci_common::IRegistry;

/// Configuration for transaction tracer
#[derive(Debug, Clone)]
pub struct TracerConfig {
    /// Registry contract address
    pub registry_address: Address,
    /// Maximum concurrent traces
    pub max_concurrent_traces: usize,
    /// Timeout for individual traces (milliseconds)
    pub trace_timeout_ms: u64,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self { registry_address: Address::ZERO, max_concurrent_traces: 10, trace_timeout_ms: 5000 }
    }
}

/// Transaction tracer that supports both txpool and block tracing
#[derive(Clone)]
pub struct UrciTracer<EthApi> {
    pub(super) eth_api: EthApi,
    pub(super) config: TracerConfig,
    pub(super) bls_handle: Arc<BlsMerkleHandle>,
    /// Cached registry config
    pub(super) cached_config: Arc<RwLock<Option<IRegistry::Config>>>,
}

/// Metrics for transaction tracer
#[derive(Debug, Default)]
pub struct TracerMetrics {
    pub traces_processed: u64,
    pub traces_failed: u64,
    pub txpool_traces: u64,
    pub block_traces: u64,
    pub invalid_registrations_found: u64,
}

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
    /// Create a new transaction tracer
    pub fn new(eth_api: EthApi, config: TracerConfig, bls_handle: BlsMerkleHandle) -> Self {
        Self {
            eth_api,
            config,
            bls_handle: Arc::new(bls_handle),
            cached_config: Arc::new(RwLock::new(None)),
        }
    }
}
