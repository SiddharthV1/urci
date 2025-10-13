//! EVM-based implementation of BLS and Merkle operations
//!
//! This module provides an EVM executor that runs actual contract code
//! to perform BLS and Merkle operations exactly as they would on-chain.
//!
//! Since BLS and Merkle operations are pure functions (stateless), we don't
//! commit transactions - we just increment nonce and reuse the same EVM state.

mod bls_ops;
mod executor;
mod merkle_ops;
mod types;
mod urc_ops;
mod worker;

pub use executor::EvmBlsMerkleExecutor;
