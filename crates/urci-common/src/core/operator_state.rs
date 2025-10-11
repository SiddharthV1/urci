//! Operator state tracking and events

use alloy_primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

use super::slashing::SlashEvent;
use super::types::RegistrationRoot;

// Complete operator state at a given block height
// This is the accumulated state from all events up to that block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorState {
    pub operator_id: uuid::Uuid,
    pub operator_address: Address,
    pub registration_root: RegistrationRoot,
    pub owner_address: Address,
    pub num_keys: Option<u32>,

    // Current registration status
    pub is_registered: bool,
    pub registration_processed: bool,

    // Current collateral (sum of all collateral events)
    pub collateral_wei: U256,

    // Block heights for key events
    pub registered_at_block: Option<u64>,
    pub unregistered_at_block: Option<u64>,
    pub slashed_at_block: Option<u64>,

    pub deleted: bool,
    pub equivocated: bool,

    // All commitment protocol relationships
    pub commitment_protocols: Vec<CommitmentProtocol>,

    // History of all events affecting this operator
    pub event_history: Vec<OperatorEvent>,

    // All slash events for this operator
    pub slash_history: Vec<SlashEvent>,
}

// Individual operator event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorEvent {
    pub block_number: u64,
    pub block_hash: B256,
    pub tx_hash: B256,
    pub tx_index: u32,
    pub log_index: u32,
    pub event_type: OperatorEventType,
    pub event_data: serde_json::Value,
    pub collateral_delta: Option<U256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperatorEventType {
    OperatorRegistered,
    OperatorSlashed,
    OperatorUnregistered,
    CollateralClaimed,
    CollateralAdded,
    OperatorOptedIn,
    OperatorOptedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentProtocol {
    pub slasher_address: Address,
    pub committer_address: Address,
    pub opted_in_at_block: Option<u64>,
    pub opted_out_at_block: Option<u64>,
    pub slashed: bool,
}
