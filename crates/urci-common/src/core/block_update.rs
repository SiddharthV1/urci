//! Single block update with events

use alloy_primitives::{BlockHash, BlockNumber};
use serde::{Deserialize, Serialize};

use super::types::WorkId;
use super::errors::SystemError;
use super::urci_tx_event_kind::UrciTxEventKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrciBlockUpdate {
    pub block_number: BlockNumber,
    pub block_hash: BlockHash,
    pub parent_block_hash: BlockHash,
    pub timestamp: u64,
    pub work_id: Option<WorkId>,
    pub parent_work_id: Option<WorkId>,
    pub events: Vec<UrciTxEventKind>,
    pub reorged: bool,
    // System error encountered during event extraction
    // If present, indexer should flip to failed state
    pub system_error: Option<SystemError>,
}

impl UrciBlockUpdate {
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }
}

// Kinds of block range updates
#[derive(Debug)]
pub enum UrciBlockRangeUpdateKind {
    Reorg {
        from: (BlockNumber, BlockHash),
        to: (BlockNumber, BlockHash),
        new_chain: super::block_range_update::UrciBlockRangeUpdate,
    },
    NewBlocks(super::block_range_update::UrciBlockRangeUpdate),
    Revert {
        from: (BlockNumber, BlockHash),
        to: (BlockNumber, BlockHash),
    },
}
