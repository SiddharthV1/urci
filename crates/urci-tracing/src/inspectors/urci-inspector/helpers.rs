//! Helper functions for tracing

use alloy_primitives::Bytes;
use super::inspector::UrciInspector;

impl UrciInspector {
    /// Get the current trace address based on call stack position
    pub(super) fn current_trace_address(&self) -> Vec<u32> {
        let mut address = Vec::new();
        for i in 0..self.call_stack.len() {
            if i < self.depth_counters.len() {
                address.push(self.depth_counters[i].saturating_sub(1));
            }
        }
        address
    }

    /// Extract function selector from input
    pub(super) fn extract_selector(input: &Bytes) -> Option<[u8; 4]> {
        if input.len() >= 4 {
            let mut selector = [0u8; 4];
            selector.copy_from_slice(&input[0..4]);
            Some(selector)
        } else {
            None
        }
    }
}
