//! Indexer task and logic
//!
//! This module contains the main indexer task that:
//! - Receives block updates from the ExEx
//! - Batches blocks for efficient DB writes
//! - Handles gaps by fetching missing blocks
//! - Processes reorgs and reverts
//! - Applies consensus finalization

mod batch_processor;
mod gap_healer;
mod handlers;
mod task;

pub use task::run_indexer;
