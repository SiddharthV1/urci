//! CallFrame type for tracking nested calls

/// Frame for tracking nested calls in the call stack
#[derive(Debug, Clone)]
pub(super) struct CallFrame {
    /// Index of this call's edge in the edges vector
    pub edge_index: usize,
    /// Trace address for this call
    #[allow(dead_code)]
    pub trace_address: Vec<u32>,
    /// Gas provided to this call
    pub gas_in: u64,
}
