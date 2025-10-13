//! EVM executor for BLS and Merkle operations

use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{Bytes, U256};
use eyre::{eyre, Result};
use reth_chainspec::{ChainSpecBuilder, HOLESKY};
use reth_db_common::init::init_genesis;
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives::public_key_to_address;
use reth_provider::test_utils::create_test_provider_factory_with_chain_spec;
use reth_revm::context::{BlockEnv, TxEnv};
use reth_revm::primitives::hardfork::SpecId;
use reth_testing_utils::generators;
use revm::context::result::ExecutionResult;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, instrument, trace};

use super::types::{ContractType, EvmCache, EvmRequest};
use super::worker;
use crate::{BLS_UTILS_LIB_ADDRESS, BLS_WRAPPER_ADDRESS, MERKLE_WRAPPER_ADDRESS};

/// EVM-based BLS and Merkle executor
pub struct EvmBlsMerkleExecutor {
    // Channel to send requests to the EVM worker
    pub(super) tx_sender: mpsc::Sender<EvmRequest>,
    // Handle to the EVM worker task (stored for cleanup)
    pub(super) _evm_handle: std::thread::JoinHandle<()>,
}

impl EvmBlsMerkleExecutor {
    /// Create new EVM executor with precompiled contracts
    /// Pre-loads everything for efficient execution
    #[instrument(skip_all)]
    pub fn new() -> Result<Self> {
        info!("Creating new EVM executor with precompiled contracts");

        // Load precompiled bytecode from build-time artifacts
        let bls_bytecode = crate::precompiled::load_bls_wrapper_bytecode()?;
        let merkle_bytecode = crate::precompiled::load_merkle_wrapper_bytecode()?;
        let bls_utils_lib = crate::precompiled::load_bls_utils_lib_bytecode()?;

        debug!(
            "Loaded precompiled contracts - BLS: {} bytes, Merkle: {} bytes",
            bls_bytecode.len(),
            merkle_bytecode.len()
        );

        Self::new_with_bytecode(bls_bytecode, merkle_bytecode, bls_utils_lib)
    }

    /// Create new EVM executor with provided bytecode
    /// Spawns an EVM worker that processes requests via channels
    pub fn new_with_bytecode(
        bls_bytecode: Bytes,
        merkle_bytecode: Bytes,
        bls_utils_lib: Bytes,
    ) -> Result<Self> {
        // Generate executor account
        let executor_keypair = generators::generate_key(&mut generators::rng());
        let executor_address = public_key_to_address(executor_keypair.public_key());

        // Create genesis with contracts and funded executor
        let mut genesis_accounts = BTreeMap::new();

        // Add BLSUtils library at genesis (must be deployed before wrappers that use it)
        genesis_accounts.insert(
            BLS_UTILS_LIB_ADDRESS,
            GenesisAccount { balance: U256::ZERO, code: Some(bls_utils_lib), ..Default::default() },
        );

        // Add BLS wrapper contract at genesis
        genesis_accounts.insert(
            BLS_WRAPPER_ADDRESS,
            GenesisAccount { balance: U256::ZERO, code: Some(bls_bytecode), ..Default::default() },
        );

        // Add Merkle wrapper contract at genesis
        genesis_accounts.insert(
            MERKLE_WRAPPER_ADDRESS,
            GenesisAccount {
                balance: U256::ZERO,
                code: Some(merkle_bytecode),
                ..Default::default()
            },
        );

        // Fund executor account
        genesis_accounts.insert(
            executor_address,
            GenesisAccount {
                balance: U256::from(100) * U256::from(10).pow(U256::from(18)), // 100 ETH
                ..Default::default()
            },
        );

        // Create chain spec with our genesis (use HOLESKY as base)
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(HOLESKY.chain)
                .genesis(Genesis { alloc: genesis_accounts, ..HOLESKY.genesis.clone() })
                .prague_activated()
                .build(),
        );

        // Create provider factory and initialize genesis
        let factory = create_test_provider_factory_with_chain_spec(chain_spec.clone());
        init_genesis(&factory)?;

        let executor_provider = EthEvmConfig::new(chain_spec.clone());

        // Pre-build all cacheable components
        let tx_template = TxEnv::builder()
            .caller(executor_address)
            .value(U256::ZERO)
            .gas_limit(10_000_000)
            .gas_price(1_000_000_000)
            .chain_id(Some(HOLESKY.chain.id())) // Match the chain spec
            .nonce(0)
            .build()
            .unwrap();

        let block_env = BlockEnv {
            number: alloy_primitives::U256::from(1),
            gas_limit: 30_000_000,
            timestamp: alloy_primitives::U256::from(12),
            ..Default::default()
        };

        let spec = SpecId::PRAGUE;

        let cache = EvmCache { block_env, spec, tx_template };

        // Create channel for sending requests to EVM worker
        let (tx_sender, rx_receiver) = mpsc::channel::<EvmRequest>(100);

        // Spawn the EVM worker thread
        let evm_handle = worker::spawn_evm_worker(factory, executor_provider, cache, rx_receiver);

        Ok(Self { tx_sender, _evm_handle: evm_handle })
    }

    /// Execute a contract call via the channel to the EVM worker
    pub(super) fn execute_call(&mut self, contract: ContractType, calldata: Bytes) -> Result<Bytes> {
        trace!("Sending EVM request via channel");

        // Create a oneshot channel for the response
        let (response_tx, response_rx) = oneshot::channel();

        // Send the request to the EVM worker
        let request = EvmRequest { contract, calldata, response_tx };

        // Use blocking send since we're in a sync context
        self.tx_sender
            .blocking_send(request)
            .map_err(|_| eyre!("Failed to send request to EVM worker"))?;

        // Wait for the response
        let result = response_rx
            .blocking_recv()
            .map_err(|_| eyre!("Failed to receive response from EVM worker"))??;

        trace!(
            "Transaction result: success={}, gas_used={:?}",
            result.result.is_success(),
            result.result.gas_used()
        );

        if result.result.is_success() {
            Ok(result.result.output().unwrap_or(&Bytes::new()).clone())
        } else {
            // Decode Solidity custom errors from revert data
            match &result.result {
                ExecutionResult::Revert { output, .. } => {
                    use alloy_sol_types::SolInterface;

                    // Try to decode as BLS error (from Solady BLS library)
                    if let Ok(bls_err) = urci_common::bindings::BLS::BLSErrors::abi_decode(output.as_ref()) {
                        return Err(eyre!("BLS error: {:?}", bls_err));
                    }

                    // Try to decode as MerkleTree error
                    if let Ok(merkle_err) = urci_common::bindings::MerkleTree::MerkleTreeErrors::abi_decode(output.as_ref()) {
                        return Err(eyre!("Merkle error: {:?}", merkle_err));
                    }

                    // Fallback for unknown errors
                    Err(eyre!("EVM reverted: 0x{}", hex::encode(output)))
                }
                _ => Err(eyre!("EVM execution failed: {:?}", result.result))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BlsOps;
    use alloy_primitives::U256;

    // Test constants from UnitTestHelper.sol
    const SECRET_KEY_1: u64 = 12345;
    const SECRET_KEY_2: u64 = 67890;

    #[test]
    fn test_evm_executor_creation() {
        let _executor = EvmBlsMerkleExecutor::new().unwrap();
    }

    #[test]
    fn test_multiple_operations_stateless() {
        // Verify that multiple operations don't interfere (stateless execution)
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();

        // First operation: generate public key
        let pk1 = executor.to_public_key(U256::from(SECRET_KEY_1)).unwrap();

        // Second operation: generate different public key
        let pk2 = executor.to_public_key(U256::from(SECRET_KEY_2)).unwrap();

        // Keys should be different
        assert!(
            pk1.x_a != pk2.x_a || pk1.x_b != pk2.x_b || pk1.y_a != pk2.y_a || pk1.y_b != pk2.y_b
        );

        // Third operation: repeat first key - should get same result (stateless)
        let pk1_again = executor.to_public_key(U256::from(SECRET_KEY_1)).unwrap();
        assert_eq!(pk1.x_a, pk1_again.x_a);
        assert_eq!(pk1.x_b, pk1_again.x_b);
        assert_eq!(pk1.y_a, pk1_again.y_a);
        assert_eq!(pk1.y_b, pk1_again.y_b);
    }
}
