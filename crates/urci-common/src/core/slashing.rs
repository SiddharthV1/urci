use alloy_primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

use super::types::RegistrationRoot;

// Types of slashing that can occur
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SlashingCall {
    Fraud {
        registration_proof: Box<crate::IRegistry::RegistrationProof>,
    },
    Commitment {
        registration_proof: Box<crate::IRegistry::RegistrationProof>,
        delegation: Box<crate::ISlasher::SignedDelegation>,
        commitment: Box<crate::ISlasher::SignedCommitment>,
    },
    SlasherCommitment {
        registration_root: RegistrationRoot,
        commitment: Box<crate::ISlasher::SignedCommitment>,
        evidence: Vec<u8>,
    },
    Equivocation {
        registration_proof: Box<crate::IRegistry::RegistrationProof>,
        delegation_one: Box<crate::ISlasher::SignedDelegation>,
        delegation_two: Box<crate::ISlasher::SignedDelegation>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashEvent {
    pub block_number: u64,
    pub block_hash: B256,
    pub tx_hash: B256,
    pub tx_index: u32,
    pub log_index: u32,
    pub sender_address: Address,
    pub sender_reward_wei: U256,
    pub burned_wei: U256,
    pub slash_type: SlashEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashEventType {
    Registration,
    Commitment { evidence: Vec<u8> },
    OffchainDelegationCommitment { evidence: Vec<u8> },
    Equivocation,
}
