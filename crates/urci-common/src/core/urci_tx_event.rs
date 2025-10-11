//! Transaction-level URCI event container

use alloy_primitives::{Address, Bytes, Log, B256, U256};
use serde::{Deserialize, Serialize};

use super::urci_event::{CallTrace, UrciEvent};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UrciTxEvent {
    pub tx_sender_address: Address,
    pub tx_to_address: Option<Address>,
    pub tx_value: U256,
    pub tx_type: u64,
    pub tx_gas: u64,
    pub tx_gas_price: u128,
    pub tx_nonce: u64,
    pub transaction_hash: B256,
    pub transaction_index: u64,
    pub transaction_input: Option<Bytes>,
    pub urc_logs: UrciIndexedLogs,
    pub urc_events: Vec<UrciEvent>,
    pub trace: Vec<CallTrace>,
    pub tracer: bool,
}

pub type UrciIndexedLogs = Vec<(Log, u64)>;
