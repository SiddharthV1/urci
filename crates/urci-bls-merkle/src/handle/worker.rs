//! Worker thread for processing BLS executor requests

use tokio::sync::mpsc;
use crate::{BlsOps, MerkleOps, UrcBlsMerkle};
use super::types::EvmThreadRequest;

/// Run the worker loop, processing requests from the channel
pub(super) fn run_worker<T>(mut rx: mpsc::Receiver<EvmThreadRequest>, mut executor: T)
where
    T: UrcBlsMerkle + BlsOps + MerkleOps,
{
    tracing::info!("BLS executor thread started");

    // Process requests forever
    while let Some(request) = rx.blocking_recv() {
        match request {
            EvmThreadRequest::VerifyRegistration {
                registrations,
                owner,
                tx,
            } => {
                let result = executor.verify_registration(&registrations, owner);
                let _ = tx.send(result);
            }
            EvmThreadRequest::VerifySignature {
                registration,
                owner,
                tx,
            } => {
                let result = executor.verify_signed_registration(&registration, owner);
                let _ = tx.send(result);
            }
            EvmThreadRequest::ToPublicKey { private_key, tx } => {
                let result = executor.to_public_key(private_key);
                let _ = tx.send(result);
            }
            EvmThreadRequest::Sign {
                message,
                private_key,
                domain_separator,
                tx,
            } => {
                let result = executor.sign(&message, private_key, &domain_separator);
                let _ = tx.send(result);
            }
            EvmThreadRequest::GenerateTree { leaves, tx } => {
                let result = executor.generate_tree(&leaves);
                let _ = tx.send(result);
            }
            EvmThreadRequest::GenerateProof { leaves, index, tx } => {
                let result = executor.generate_proof(&leaves, index);
                let _ = tx.send(result);
            }
            EvmThreadRequest::Shutdown => {
                tracing::info!("BLS executor thread shutting down");
                break;
            }
        }
    }

    tracing::info!("BLS executor thread terminated");
}
