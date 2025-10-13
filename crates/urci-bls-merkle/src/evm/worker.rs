//! EVM worker thread for executing contract calls

use alloy_primitives::TxKind;
use reth_evm::{ConfigureEvm, Evm};
use reth_evm_ethereum::EthEvmConfig;
use reth_provider::{DatabaseProviderFactory, ProviderFactory};
use reth_provider::test_utils::MockNodeTypesWithDB;
use reth_revm::database::StateProviderDatabase;
use revm::database::CacheDB;
use revm_inspector::NoOpInspector;
use tokio::sync::mpsc;

use super::types::{ContractType, EvmCache, EvmRequest};
use crate::{BLS_WRAPPER_ADDRESS, MERKLE_WRAPPER_ADDRESS};

/// Spawn the EVM worker thread that processes execution requests
pub(super) fn spawn_evm_worker(
    factory: ProviderFactory<MockNodeTypesWithDB>,
    executor_provider: EthEvmConfig,
    cache: EvmCache,
    mut rx_receiver: mpsc::Receiver<EvmRequest>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // Set up EVM environment in the worker thread
        let fake_header = reth_primitives::Header {
            number: 1,
            gas_limit: 30_000_000,
            timestamp: 1,
            excess_blob_gas: Some(0),
            ..Default::default()
        };

        // Create database
        let provider = factory.database_provider_ro().expect("Failed to create provider");
        let state_db = StateProviderDatabase::new(provider.latest());
        let mut cache_db = CacheDB::new(state_db);

        let evm_env = executor_provider.evm_env(&fake_header).expect("evm_env should not fail");

        // Create EVM without tracing (use NoOpInspector for no output)
        let inspector = NoOpInspector;
        let mut evm = executor_provider.evm_with_env_and_inspector(
            &mut cache_db,
            evm_env.clone(),
            inspector,
        );

        // Process requests - use blocking recv since we're in a thread
        while let Some(request) = rx_receiver.blocking_recv() {
            // Determine target address based on contract type
            let to = match request.contract {
                ContractType::Bls => BLS_WRAPPER_ADDRESS,
                ContractType::Merkle => MERKLE_WRAPPER_ADDRESS,
            };

            // Build transaction
            let mut tx = cache.tx_template.clone();
            tx.kind = TxKind::Call(to);
            tx.data = request.calldata.clone();
            tx.nonce = 0;

            // Execute
            let result = evm.transact(tx);

            // Send back the result
            let _ = request.response_tx.send(result);
        }
    })
}
