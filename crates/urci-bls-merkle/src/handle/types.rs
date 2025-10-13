//! Request types for BLS executor thread communication

use alloy_primitives::{Address, B256, U256};
use eyre::Result;
use tokio::sync::oneshot;
use urci_common::{BLS, IRegistry, RegistrationValidationResult};

/// Request types sent to the EVM executor thread
pub enum EvmThreadRequest {
    /// Verify a batch of registrations
    VerifyRegistration {
        registrations: Vec<IRegistry::SignedRegistration>,
        owner: Address,
        tx: oneshot::Sender<Result<RegistrationValidationResult>>,
    },
    /// Verify a single signature
    VerifySignature {
        registration: Box<IRegistry::SignedRegistration>,
        owner: Address,
        tx: oneshot::Sender<Result<bool>>,
    },
    /// Convert private key to public key
    ToPublicKey {
        private_key: U256,
        tx: oneshot::Sender<Result<BLS::G1Point>>,
    },
    /// Sign a message
    Sign {
        message: Vec<u8>,
        private_key: U256,
        domain_separator: Vec<u8>,
        tx: oneshot::Sender<Result<BLS::G2Point>>,
    },
    /// Generate merkle tree
    GenerateTree {
        leaves: Vec<B256>,
        tx: oneshot::Sender<Result<B256>>,
    },
    /// Generate merkle proof
    GenerateProof {
        leaves: Vec<B256>,
        index: usize,
        tx: oneshot::Sender<Result<Vec<B256>>>,
    },
    /// Shutdown the executor thread
    Shutdown,
}
