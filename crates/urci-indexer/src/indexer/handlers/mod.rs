//! Update handlers for different types of chain updates

pub mod new_blocks;
pub mod reorg;
pub mod revert;

pub use new_blocks::handle_new_blocks;
pub use reorg::handle_reorg;
pub use revert::handle_revert;
