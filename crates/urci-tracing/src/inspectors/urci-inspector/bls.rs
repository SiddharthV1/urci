//! BLS signature validation for operator registrations

use alloy_primitives::Log;
use alloy_sol_types::{SolCall, SolEventInterface};
use urci_common::Registry::registerCall;
use urci_common::{RegistryEvents, Verifier};

use super::inspector::UrciInspector;

impl UrciInspector {
    /// Process a log emitted by the Registry contract
    /// This is where we validate that a proper URC event was emitted
    pub(super) fn process_registry_log(&mut self, log: &Log) {
        // Try to decode as a valid RegistryEvent
        let decoded_event = match RegistryEvents::decode_raw_log(log.topics(), &log.data.data) {
            Ok(event) => event,
            Err(_) => {
                // Not a valid URC event, ignore
                return;
            }
        };

        // We have a valid URC event!
        self.has_valid_urc_events = true;

        // For OperatorRegistered events, do BLS validation
        if let RegistryEvents::OperatorRegistered(_data) = decoded_event {
            // Get the call input we stored earlier
            let call_input = match &self.pending_registry_call_input {
                Some(input) => input,
                None => {
                    tracing::warn!(
                        "OperatorRegistered event but no pending call input for tx {:?}",
                        self.tx_hash
                    );
                    return;
                }
            };

            // Decode the register() call
            let register_call = match registerCall::abi_decode(call_input) {
                Ok(call) => call,
                Err(e) => {
                    tracing::error!(
                        "Failed to decode register call for tx {:?}: {}",
                        self.tx_hash,
                        e
                    );
                    return;
                }
            };

            // Validate all registrations at once using BLS executor
            if let Some(ref handle) = self.bls_handle {
                match handle.verify_registration_blocking(register_call.registrations.clone(), register_call.owner) {
                    Ok(result) => {
                        // Store the FULL validation result (includes tree with root and leaves)
                        // This is what the database needs for inserting:
                        // - merkle_tree table (root + leaves)
                        // - merkle_inclusion_proof table (proofs for each key)
                        // - operators table (registration_root = tree.root)

                        // Log any fraudulent registrations found
                        for (idx, validation) in result.signature.iter().enumerate() {
                            if validation.is_fraudulent {
                                let registration_root = result.tree.leaves.get(idx)
                                    .copied()
                                    .unwrap_or_default();

                                tracing::warn!(
                                    "Found slashable registration in block {}: tx={:?}, root={:?}, owner={:?}, error={:?}",
                                    self.block_number,
                                    self.tx_hash,
                                    registration_root,
                                    register_call.owner,
                                    validation.validation_error
                                );
                            }
                        }

                        self.registration_validations.push(result);
                    }
                    Err(e) => {
                        tracing::error!(
                            "BLS verification failed for tx {:?} in block {}: {}",
                            self.tx_hash,
                            self.block_number,
                            e
                        );
                    }
                }
            } else {
                tracing::debug!("No BLS handle available for validation");
            }
        }
    }
}
