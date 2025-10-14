//! Registry configuration fetching

use alloy_network::TransactionBuilder;
use alloy_rpc_types_eth::{BlockId, BlockNumberOrTag};
use alloy_sol_types::SolCall;
use eyre::Result;
use reth_rpc_eth_api::helpers::{FullEthApi, LoadBlock};
use reth_storage_api::{BlockReader, ReceiptProvider};
use revm::context_interface::result::ExecutionResult;
use tracing::{debug, error, info, instrument};
use urci_common::IRegistry;

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
    /// Get registry configuration from chain
    #[instrument(skip(self), fields(registry = %self.config.registry_address))]
    pub async fn get_config(&self) -> Result<IRegistry::Config> {
        // Check cache first
        {
            let cache = self.cached_config.read().await;
            if let Some(config) = cache.as_ref() {
                debug!("Returning cached registry config");
                return Ok(config.clone());
            }
        }

        debug!("Fetching registry config from chain");

        // Fetch from chain using eth_call
        let config_call = urci_common::Registry::getConfigCall {};
        let call_data = config_call.abi_encode();

        // Use spawn_with_call_at to execute the call and get result
        let eth_api = self.eth_api.clone();
        let tx_request = alloy_rpc_types_eth::TransactionRequest::default()
            .with_to(self.config.registry_address)
            .input(call_data.into());

        let result_and_state = self
            .eth_api
            .spawn_with_call_at(
                tx_request.into(),
                BlockId::Number(BlockNumberOrTag::Latest),
                Default::default(),
                move |db, evm_env, tx_env| {
                    // Execute the transaction using transact method
                    eth_api.transact(db, evm_env, tx_env)
                },
            )
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to execute eth_call for getConfig");
                e
            })?;

        // Extract the output bytes from execution result
        let result = match result_and_state.result {
            ExecutionResult::Success { output, .. } => {
                let data = output.into_data();
                error!("✓ getConfig eth_call returned Success with {} bytes: 0x{}", data.len(), hex::encode(&data));
                data
            },
            ExecutionResult::Revert { output, .. } => {
                error!(revert_output = ?output, "eth_call reverted for getConfig");
                return Err(eyre::eyre!("eth_call reverted: {:?}", output));
            },
            ExecutionResult::Halt { reason, .. } => {
                error!(halt_reason = ?reason, "eth_call halted for getConfig");
                return Err(eyre::eyre!("eth_call halted: {:?}", reason));
            },
        };

        error!("Attempting to decode {} bytes as Config tuple: 0x{}", result.len(), hex::encode(&result));

        let config = urci_common::Registry::getConfigCall::abi_decode_returns(&result)
            .map_err(|e| {
                error!(error = %e, "Failed to decode getConfig return value");
                error!("Expected format: (uint80, uint32, uint32, uint32, uint32) = {} bytes minimum", 32 * 5);
                error!("Raw bytes: 0x{}", hex::encode(&result));
                e
            })?;

        // Update cache
        {
            let mut cache = self.cached_config.write().await;
            *cache = Some(config.clone());
        }

        info!("✓ Fetched registry config: fraud window = {}", config.fraudProofWindow);

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Uint;
    use alloy_sol_types::{SolCall, SolType};
    use urci_common::{IRegistry, Registry};

    #[test]
    fn test_config_struct_decoding() {
        // Create a test config
        let config = IRegistry::Config {
            minCollateralWei: Uint::<80, 2>::from(1_000_000_000_000_000_000u64), // 1 ETH
            fraudProofWindow: 172800,  // 2 days in seconds
            unregistrationDelay: 604800, // 7 days
            slashWindow: 86400,        // 1 day
            optInDelay: 3600,          // 1 hour
        };

        // Encode config as ABI return data
        let encoded_return = <IRegistry::Config as SolType>::abi_encode(&config);

        // Test decoding
        let decoded = Registry::getConfigCall::abi_decode_returns(&encoded_return).unwrap();

        assert_eq!(decoded.minCollateralWei, config.minCollateralWei);
        assert_eq!(decoded.fraudProofWindow, config.fraudProofWindow);
        assert_eq!(decoded.unregistrationDelay, config.unregistrationDelay);
        assert_eq!(decoded.slashWindow, config.slashWindow);
        assert_eq!(decoded.optInDelay, config.optInDelay);
    }
}
