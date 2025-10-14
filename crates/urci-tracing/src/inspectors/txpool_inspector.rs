//! UrcTxPoolInspector - Lightweight BLS validation for txpool transactions
//!
//! This inspector only validates BLS signatures for pending transactions.
//! It does NOT capture call graphs (too expensive for txpool).
//! It DOES track if the registry call succeeded.

use alloy_primitives::{Address, Bytes, B256};
use alloy_sol_types::SolCall;
use revm::inspector::Inspector;
use revm::interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CreateInputs, CreateOutcome,
};

use urci_common::Registry::registerCall;
use urci_common::Verifier;

/// Lightweight inspector for txpool BLS validation only
pub struct UrcTxPoolInspector {
    /// Registry address to monitor
    registry_address: Address,

    /// BLS validation results (full results with merkle tree data)
    validation_results: Vec<urci_common::RegistrationValidationResult>,

    /// Track if registry call succeeded (CRITICAL: don't index failed txs)
    registry_call_succeeded: bool,

    /// Shared BLS handle for validation
    bls_handle: Option<urci_bls_merkle::BlsMerkleHandle>,

    /// Transaction context
    tx_hash: B256,
}

impl UrcTxPoolInspector {
    /// Create a new txpool inspector
    pub fn new(registry_address: Address, tx_hash: B256) -> Self {
        Self {
            registry_address,
            validation_results: Vec::new(),
            registry_call_succeeded: false,
            bls_handle: None,
            tx_hash,
        }
    }

    /// Set the shared BLS handle
    pub fn with_bls_handle(mut self, handle: urci_bls_merkle::BlsMerkleHandle) -> Self {
        self.bls_handle = Some(handle);
        self
    }

    /// Get validation results
    pub fn get_validation_results(&self) -> Vec<urci_common::RegistrationValidationResult> {
        self.validation_results.clone()
    }

    /// Check if registry call succeeded
    pub fn registry_call_succeeded(&self) -> bool {
        self.registry_call_succeeded
    }

    /// Process a call to the Registry contract
    fn process_registry_call(&mut self, input: &Bytes) {
        // Try to decode as register() call
        if let Ok(call) = registerCall::abi_decode(input) {
            // Validate all registrations at once using BLS executor
            if let Some(ref handle) = self.bls_handle {
                match handle.verify_registration_blocking(call.registrations.clone(), call.owner) {
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
                                    "Found slashable registration in txpool: tx={:?}, root={:?}, owner={:?}, error={:?}",
                                    self.tx_hash,
                                    registration_root,
                                    call.owner,
                                    validation.validation_error
                                );
                            }
                        }

                        self.validation_results.push(result);
                    }
                    Err(e) => {
                        tracing::error!(
                            "BLS verification failed for txpool tx {:?}: {}",
                            self.tx_hash,
                            e
                        );
                    }
                }
            } else {
                tracing::debug!("No BLS handle available for txpool validation");
            }
        }
    }
}

// Implement the Inspector trait for revm
impl<CTX> Inspector<CTX, EthInterpreter> for UrcTxPoolInspector
where
    CTX: revm::context_interface::ContextTr,
{
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        // Check if this is a call to the Registry
        if inputs.target_address == self.registry_address {
            let input_bytes = inputs.input.bytes(context);
            self.process_registry_call(&input_bytes);
        }

        None // Don't override the call
    }

    fn call_end(&mut self, _context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        // CRITICAL: Track if registry call succeeded
        if inputs.target_address == self.registry_address {
            self.registry_call_succeeded = outcome.result.result.is_ok();

            if !self.registry_call_succeeded {
                tracing::warn!(
                    "Registry call failed in txpool: tx={:?}, gas_used={}",
                    self.tx_hash,
                    outcome.result.gas.spent()
                );
            }
        }
    }

    // Don't care about creates in txpool
    fn create(&mut self, _context: &mut CTX, _inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        None
    }

    fn create_end(
        &mut self,
        _context: &mut CTX,
        _inputs: &CreateInputs,
        _outcome: &mut CreateOutcome,
    ) {
        // No-op
    }
}
