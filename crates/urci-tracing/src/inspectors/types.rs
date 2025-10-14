//! Types for call tracing (extracted from call_graph.rs)

use alloy_primitives::{Address, Bytes, B256, U256};
use revm::interpreter::CallScheme;

/// Type of call in the EVM
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CallType {
    Call,
    DelegateCall,
    StaticCall,
    CallCode,
    Create,
    Create2,
}

impl From<CallScheme> for CallType {
    fn from(scheme: CallScheme) -> Self {
        match scheme {
            CallScheme::Call => CallType::Call,
            CallScheme::DelegateCall => CallType::DelegateCall,
            CallScheme::StaticCall => CallType::StaticCall,
            CallScheme::CallCode => CallType::CallCode,
        }
    }
}

/// Status of a call
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CallStatus {
    Success,
    Revert,
    Halt,
}

/// Flat edge record for database storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CallEdge {
    pub id: Option<uuid::Uuid>,
    pub block_number: u64,
    pub block_hash: B256,
    pub tx_hash: B256,
    pub tx_index: u32,
    pub trace_address: Vec<u32>,
    pub depth: u32,
    pub call_type: CallType,
    pub from_addr: Address,
    pub to_addr: Address,
    pub code_addr: Option<Address>,
    pub value_wei: U256,
    pub gas_in: u64,
    pub gas_used: u64,
    pub status: CallStatus,
    pub selector: Option<[u8; 4]>,
    pub input_data: Option<Bytes>,
    pub output_data: Option<Bytes>,
    pub input_len: usize,
    pub output_len: usize,
    pub error_msg: Option<String>,
}

/// Transaction trace data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionTrace {
    pub tx_hash: B256,
    pub block_number: u64,
    pub block_hash: B256,
    pub tx_index: u32,
    pub edges: Vec<CallEdge>,
    pub has_urc_events: bool,
    pub registration_validations: Vec<urci_common::RegistrationValidationResult>,
    pub gas_used: u64,
    pub success: bool,
}
