//! URC Inspector - Transaction tracing with BLS validation
//!
//! This inspector captures complete transaction execution traces including:
//! - Full call graph with internal calls
//! - BLS signature validation for operator registrations
//! - Registry event detection and enrichment
//! - Log-to-call mapping for event context

mod api;
mod bls;
mod call_frame;
mod helpers;
mod revm_inspector;
mod inspector;

pub use inspector::UrciInspector;

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, B256};

    #[test]
    fn test_new_inspector() {
        let registry = Address::ZERO;
        let tx_hash = B256::ZERO;
        let inspector = UrciInspector::new(registry, tx_hash, 100, B256::ZERO, 0);

        assert!(!inspector.has_urc_events());
        assert_eq!(inspector.get_log_to_call_map().len(), 0);
    }

    #[test]
    fn test_builder_pattern() {
        let registry = Address::ZERO;
        let tx_hash = B256::ZERO;

        let inspector = UrciInspector::for_tracing(registry, tx_hash, 100, B256::ZERO, 0)
            .with_config(false, 2048);

        assert!(!inspector.has_urc_events());
    }
}
