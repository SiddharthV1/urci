//! Public accessor methods

use super::inspector::UrciInspector;
use crate::inspectors::types::{CallEdge, TransactionTrace};
use urci_common::RegistrationValidationResult;

impl UrciInspector {
    /// Get the collected edges
    pub fn get_edges(self) -> Vec<CallEdge> {
        self.edges
    }

    /// Get validation results
    pub fn get_validation_results(&self) -> Vec<RegistrationValidationResult> {
        self.registration_validations.clone()
    }

    /// Get log to call mapping for enriching URC events
    pub fn get_log_to_call_map(&self) -> &[(u64, u32, u32)] {
        &self.log_to_call_map
    }

    /// Check if valid URC events were detected during tracing
    pub fn has_urc_events(&self) -> bool {
        self.has_valid_urc_events
    }

    /// Build the transaction trace
    pub fn into_trace(self) -> TransactionTrace {
        TransactionTrace {
            tx_hash: self.tx_hash,
            block_number: self.block_number,
            block_hash: self.block_hash,
            tx_index: self.tx_index,
            edges: self.edges,
            has_urc_events: self.has_valid_urc_events,
            registration_validations: self.registration_validations.clone(),
            gas_used: 0,   // Will be set by trace processor
            success: true, // Will be set by trace processor
        }
    }
}
