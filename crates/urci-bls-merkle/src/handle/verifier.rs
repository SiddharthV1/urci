//! Verifier trait implementation for BlsMerkleHandle

use alloy_primitives::{Address, B256, U256};
use async_trait::async_trait;
use eyre::Result;
use tokio::sync::oneshot;
use urci_common::{BLS, IRegistry, RegistrationValidationResult, Verifier};
use super::client::BlsMerkleHandle;
use super::types::EvmThreadRequest;

/// Implement the Verifier trait for BlsMerkleHandle
#[async_trait]
impl Verifier for BlsMerkleHandle {
    /// Verify a batch of registrations (async)
    async fn verify_registration(
        &self,
        registrations: Vec<IRegistry::SignedRegistration>,
        owner: Address,
    ) -> Result<RegistrationValidationResult> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(EvmThreadRequest::VerifyRegistration {
                registrations,
                owner,
                tx,
            })
            .await
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.await
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }

    /// Verify a batch of registrations (blocking)
    fn verify_registration_blocking(
        &self,
        registrations: Vec<IRegistry::SignedRegistration>,
        owner: Address,
    ) -> Result<RegistrationValidationResult> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .blocking_send(EvmThreadRequest::VerifyRegistration {
                registrations,
                owner,
                tx,
            })
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.blocking_recv()
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }

    /// Verify a single signed registration (async)
    async fn verify_signed_registration(
        &self,
        registration: IRegistry::SignedRegistration,
        owner: Address,
    ) -> Result<bool> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(EvmThreadRequest::VerifySignature {
                registration: Box::new(registration),
                owner,
                tx,
            })
            .await
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.await
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }

    /// Verify a single signed registration (blocking)
    fn verify_signed_registration_blocking(
        &self,
        registration: IRegistry::SignedRegistration,
        owner: Address,
    ) -> Result<bool> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .blocking_send(EvmThreadRequest::VerifySignature {
                registration: Box::new(registration),
                owner,
                tx,
            })
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.blocking_recv()
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }

    /// Convert private key to public key
    async fn to_public_key(&self, private_key: U256) -> Result<BLS::G1Point> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(EvmThreadRequest::ToPublicKey { private_key, tx })
            .await
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.await
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }

    /// Sign a message
    async fn sign(
        &self,
        message: Vec<u8>,
        private_key: U256,
        domain_separator: Vec<u8>,
    ) -> Result<BLS::G2Point> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(EvmThreadRequest::Sign {
                message,
                private_key,
                domain_separator,
                tx,
            })
            .await
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.await
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }

    /// Generate merkle tree root
    async fn generate_tree(&self, leaves: Vec<B256>) -> Result<B256> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(EvmThreadRequest::GenerateTree { leaves, tx })
            .await
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.await
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }

    /// Generate merkle proof
    async fn generate_proof(&self, leaves: Vec<B256>, index: usize) -> Result<Vec<B256>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(EvmThreadRequest::GenerateProof { leaves, index, tx })
            .await
            .map_err(|_| eyre::eyre!("EVM executor thread is dead"))?;

        rx.await
            .map_err(|_| eyre::eyre!("EVM executor thread dropped response channel"))?
    }
}
