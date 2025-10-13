use alloy_primitives::{BlockHash, BlockNumber};
use urci_common::{WorkId, UrciBatchedBlockRangeUpdate};

#[derive(Debug, Clone)]
pub struct LastUpdate {
    pub block_number: BlockNumber,
    pub block_hash: BlockHash,
    pub work_id: WorkId,
    pub timestamp: u64,
}

pub struct DBState {
    pub session_tip: LastUpdate,
    pub indexer_tip: LastUpdate,
    pub finalized: LastUpdate,
    pub is_syncing: bool,
    pub session_id: Option<uuid::Uuid>,
}

impl DBState {
    pub fn is_valid_update(&self, update: &UrciBatchedBlockRangeUpdate) -> bool {
        if update.blocks.is_empty() {
            return false;
        }

        // Check batch follows the current tip
        if update.from != self.indexer_tip.block_number + 1 {
            return false;
        }

        // Check first block's parent matches current tip
        if let Some(first_block) = update.blocks.first() {
            if let Some(parent_work_id) = first_block.parent_work_id {
                if parent_work_id != self.indexer_tip.work_id {
                    return false;
                }
            }
        }

        // Don't accept blocks before finalized
        if update.from < self.finalized.block_number {
            return false;
        }

        true
    }

    pub fn is_missing_blocks(&self, update: &UrciBatchedBlockRangeUpdate) -> bool {
        if update.blocks.is_empty() {
            return false;
        }

        // Missing blocks if gap exists or parent doesn't match
        update.from > self.indexer_tip.block_number + 1
            || (update.blocks.first()
                .and_then(|b| b.parent_work_id)
                != Some(self.indexer_tip.work_id))
    }
}
