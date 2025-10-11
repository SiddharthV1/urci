//! Batched block range updates for database writes

use alloy_primitives::BlockNumber;

use super::block_update::UrciBlockUpdate;

// Batched version of block range update - for DB writes
#[derive(Debug, Clone)]
pub struct UrciBatchedBlockRangeUpdate {
    pub from: BlockNumber,
    pub to: BlockNumber,
    pub blocks: Vec<UrciBlockUpdate>,
    pub has_events: bool,
}

impl UrciBatchedBlockRangeUpdate {
    pub fn new() -> Self {
        Self {
            from: 0,
            to: 0,
            blocks: Vec::new(),
            has_events: false,
        }
    }

    pub fn add_block(&mut self, block: UrciBlockUpdate) {
        if self.blocks.is_empty() {
            // First block sets the range start
            self.from = block.block_number;
            self.to = block.block_number;
        } else {
            // Update range end
            self.to = block.block_number;
        }

        if block.has_events() {
            self.has_events = true;
        }

        self.blocks.push(block);
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}

impl Default for UrciBatchedBlockRangeUpdate {
    fn default() -> Self {
        Self::new()
    }
}
