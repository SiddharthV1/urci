use alloy_primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};

use super::urci_event_kind::UrciEventKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrciEvent {
    pub caller_address: Address,
    pub call_depth: u8,
    pub call_index_at_depth: u8,
    pub input: Option<Bytes>,
    pub log_index: u64,
    pub event: UrciEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallTrace;
