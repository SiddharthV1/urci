//! URC Tracing - Transaction tracing with BLS validation for URC indexer
//!
//! This crate provides:
//! - Registry config fetching from chain
//! - Transaction metadata extraction and enrichment
//! - BLS signature validation for URC registrations
//! - Call graph extraction for transactions

use eyre::Result;
use urci_common::{IRegistry, UrciBlockUpdate};

pub mod inspectors;
pub mod tracer;

// Re-export main components
pub use inspectors::{
    CallEdge, CallStatus, CallType, TransactionTrace, UrcTxPoolInspector, UrciInspector,
};
pub use tracer::{TracerConfig, TracerMetrics, UrciTracer};

/// Main tracer trait - defines the interface for block enrichment
#[async_trait::async_trait]
pub trait Tracer: Send + Sync {
    /// Get registry configuration from chain
    async fn get_config(&self) -> Result<IRegistry::Config>;

    /// Main job - enrich blocks with metadata (call graphs, BLS validation, etc.)
    async fn extract_urc_metadata(&self, block: UrciBlockUpdate) -> Result<UrciBlockUpdate>;

    /// Handle missing blocks when DB is behind
    async fn trace_missing_blocks(&self, from: u64, to: u64) -> Result<Vec<UrciBlockUpdate>>;
}
