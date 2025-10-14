//! Tracer and registry config initialization

use alloy_primitives::Address;
use eyre::Result;
use std::time::Duration;
use tracing::{info, warn};
use urci_tracing::UrciTracer;

/// Initialize tracer and fetch registry config from on-chain contract
///
/// Retries with exponential backoff if contract not deployed yet (E2E scenarios)
pub async fn initialize_tracer<EthApi>(
    eth_api: EthApi,
    registry_address: Address,
    bls_handle: urci_bls_merkle::BlsMerkleHandle,
) -> Result<(UrciTracer<EthApi>, urci_common::IRegistry::Config)>
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
    info!("Initializing tracer");

    let tracer_config = urci_tracing::TracerConfig {
        registry_address,
        max_concurrent_traces: 10,
        trace_timeout_ms: 5000,
    };

    let tracer = UrciTracer::new(eth_api, tracer_config, bls_handle);

    info!(
        "Fetching registry config from contract at {}",
        registry_address
    );

    let registry_config = fetch_registry_config_with_retry(&tracer).await?;

    info!("✓ Tracer initialized and registry config loaded");

    Ok((tracer, registry_config))
}

/// Fetch registry config with exponential backoff retry logic
///
/// This is necessary in E2E test scenarios where the registry contract
/// may not be deployed yet when the indexer starts.
async fn fetch_registry_config_with_retry<EthApi>(
    tracer: &UrciTracer<EthApi>,
) -> Result<urci_common::IRegistry::Config>
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
    let mut retry_count = 0;
    let max_retries = 10;
    let mut delay = Duration::from_secs(2);

    loop {
        match tracer.get_config().await {
            Ok(cfg) => {
                info!("✓ Registry config loaded successfully");
                return Ok(cfg);
            }
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                warn!(
                    "Failed to fetch registry config (attempt {}/{}): {}. Retrying in {:?}...",
                    retry_count, max_retries, e, delay
                );
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(30));
            }
            Err(e) => {
                return Err(eyre::eyre!(
                    "Failed to fetch registry config after {} attempts: {}. \
                     Ensure the registry contract is deployed at the configured address",
                    max_retries,
                    e
                ));
            }
        }
    }
}
