//! Internal types for EVM execution

use alloy_primitives::Bytes;
use reth_evm_ethereum::EthEvm;
use reth_evm::precompiles::PrecompilesMap;
use reth_provider::StateProvider;
use reth_revm::database::StateProviderDatabase;
use reth_revm::context::{BlockEnv, TxEnv};
use reth_revm::primitives::hardfork::SpecId;
use revm::database::CacheDB;
use revm::context::result::{EVMError, ExecResultAndState, ExecutionResult};
use revm::inspector::NoOpInspector;
use revm_inspector::inspectors::TracerEip3155;
use reth_provider::ProviderError;
use tokio::sync::oneshot;

/// Pre-cached EVM configuration
#[allow(dead_code)]
pub(super) struct EvmCache {
    pub(super) block_env: BlockEnv,
    pub(super) spec: SpecId,
    pub(super) tx_template: TxEnv,
}

/// Contract type for EVM execution
pub(super) enum ContractType {
    Bls,
    Merkle,
}

/// Message sent to the EVM worker thread
pub(super) struct EvmRequest {
    pub(super) contract: ContractType,
    pub(super) calldata: Bytes,
    pub(super) response_tx:
        oneshot::Sender<Result<ExecResultAndState<ExecutionResult>, EVMError<ProviderError>>>,
}

/// Type alias for EVM with tracer
#[allow(dead_code)]
pub(super) type EvmWithTracer<'a> = EthEvm<
    &'a mut CacheDB<StateProviderDatabase<Box<dyn StateProvider>>>,
    TracerEip3155,
    PrecompilesMap,
>;

/// Type alias for EVM without tracer
#[allow(dead_code)]
pub(super) type EvmNoTracer<'a> = EthEvm<
    &'a mut CacheDB<StateProviderDatabase<Box<dyn StateProvider>>>,
    NoOpInspector,
    PrecompilesMap,
>;
