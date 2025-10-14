//! Node initialization and orchestration
//!
//! This module handles the complete node initialization flow:
//! - Database setup and migrations
//! - BLS executor launch
//! - Node + ExEx installation
//! - Tracer initialization
//! - TxPool monitoring
//! - Indexer task spawning

mod builder;
mod db_init;
mod tracer_init;
mod txpool_monitor;

pub use builder::build_and_launch;
