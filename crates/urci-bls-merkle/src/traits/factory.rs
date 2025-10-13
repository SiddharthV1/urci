//! Executor factory for creating BLS/Merkle implementations
//!
//! Provides factory functions to create different executor types
//! (EVM-based or native implementations).

use eyre::Result;

use super::urc_bls_merkle::UrcBlsMerkle;

/// Factory for creating executors
pub enum ExecutorType {
    /// Use EVM-based execution with deployed contracts
    Evm,
    /// Use native Rust implementation
    Native,
}

/// Create an executor based on the type
pub fn create_executor(executor_type: ExecutorType) -> Result<Box<dyn UrcBlsMerkle + Send + Sync>> {
    match executor_type {
        ExecutorType::Evm => {
            use crate::evm::EvmBlsMerkleExecutor;
            use crate::cached_executor::CachedBlsExecutor;
            let inner = EvmBlsMerkleExecutor::new()?;
            Ok(Box::new(CachedBlsExecutor::new(inner)))
        },
        ExecutorType::Native => {
            // Will be implemented when native executor is ready
            Err(eyre::eyre!("Native executor not yet implemented"))
        },
    }
}
