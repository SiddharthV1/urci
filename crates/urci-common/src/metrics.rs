//! Metrics module for URC indexer components.

use eyre::Result;
use std::time::Duration;
use tracing::info;

// Core metrics for the indexer
pub struct IndexerMetrics {
    pub service_name: String,
}

impl IndexerMetrics {
    // Create new metrics instance
    pub fn new(service_name: &str) -> Result<Self> {
        Ok(Self {
            service_name: service_name.to_string(),
        })
    }

    // Record a processed block
    pub fn record_block_processed(&self, chain_id: u64, block_number: u64, events: u64) {
        // Metrics recording implementation
        info!(
            service = %self.service_name,
            chain_id = chain_id,
            block_number = block_number,
            events = events,
            "Block processed"
        );
    }

    // Record database write
    pub fn record_db_write(&self, table: &str, duration: Duration, records: u64) {
        info!(
            service = %self.service_name,
            table = table,
            duration_ms = duration.as_millis(),
            records = records,
            "Database write completed"
        );
    }

    // Record a reorg
    pub fn record_reorg(&self, chain_id: u64, depth: u64) {
        info!(
            service = %self.service_name,
            chain_id = chain_id,
            depth = depth,
            "Reorg detected"
        );
    }
}

// Service-specific metrics
pub struct ServiceMetrics {
    pub service_name: String,
}

impl ServiceMetrics {
    // Create new service metrics
    pub fn new(service_name: &str) -> Result<Self> {
        Ok(Self {
            service_name: service_name.to_string(),
        })
    }

    // Record a request
    pub fn record_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        info!(
            service = %self.service_name,
            method = method,
            path = path,
            status = status,
            duration_ms = duration.as_millis(),
            "Request processed"
        );
    }
}
