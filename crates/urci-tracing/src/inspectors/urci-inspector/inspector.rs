//! Main UrciInspector struct and constructors

use alloy_primitives::{Address, Bytes, B256};
use urci_common::RegistrationValidationResult;
use crate::inspectors::types::CallEdge;
use super::call_frame::CallFrame;

/// Transaction tracing inspector for URC transactions
pub struct UrciInspector {
    /// Registry address to monitor
    pub(super) registry_address: Address,

    /// Call graph data - tracks all EVM calls
    pub(super) call_stack: Vec<CallFrame>,
    pub(super) edges: Vec<CallEdge>,
    pub(super) depth_counters: Vec<u32>,

    /// BLS validation results - stores FULL validation results including merkle tree data
    pub(super) registration_validations: Vec<RegistrationValidationResult>,

    /// Pending call input from registry calls (stored when call happens, used when log is emitted)
    pub(super) pending_registry_call_input: Option<Bytes>,

    /// Track if any valid URC events were emitted
    pub(super) has_valid_urc_events: bool,

    /// Map log index -> (call_depth, call_index_at_depth) for URC event enrichment
    pub(super) log_to_call_map: Vec<(u64, u32, u32)>,

    /// Transaction context
    pub(super) tx_hash: B256,
    pub(super) block_number: u64,
    pub(super) block_hash: B256,
    pub(super) tx_index: u32,

    /// Configuration
    pub(super) store_full_data: bool,
    pub(super) max_data_size: usize,

    /// Shared BLS handle for validation
    pub(super) bls_handle: Option<urci_bls_merkle::BlsMerkleHandle>,
}

impl UrciInspector {
    /// Create a new inspector for a specific registry
    pub fn new(
        registry_address: Address,
        tx_hash: B256,
        block_number: u64,
        block_hash: B256,
        tx_index: u32,
    ) -> Self {
        Self {
            registry_address,
            call_stack: Vec::with_capacity(8),
            edges: Vec::new(),
            depth_counters: vec![0],
            registration_validations: Vec::new(),
            pending_registry_call_input: None,
            has_valid_urc_events: false,
            log_to_call_map: Vec::new(),
            tx_hash,
            block_number,
            block_hash,
            tx_index,
            store_full_data: true,
            max_data_size: 4096,
            bls_handle: None,
        }
    }

    /// Create inspector for transaction tracing with full call graph and BLS validation
    pub fn for_tracing(
        registry_address: Address,
        tx_hash: B256,
        block_number: u64,
        block_hash: B256,
        tx_index: u32,
    ) -> Self {
        Self::new(
            registry_address,
            tx_hash,
            block_number,
            block_hash,
            tx_index,
        )
    }

    /// Set the shared BLS handle
    pub fn with_bls_handle(mut self, handle: urci_bls_merkle::BlsMerkleHandle) -> Self {
        self.bls_handle = Some(handle);
        self
    }

    /// Configure data storage
    pub fn with_config(mut self, store_full_data: bool, max_data_size: usize) -> Self {
        self.store_full_data = store_full_data;
        self.max_data_size = max_data_size;
        self
    }
}
