//! Transaction tracer - orchestrates tracing for both txpool and block transactions
//!
//! This module provides:
//! - Tracing for pending txpool transactions (pre-validation)
//! - Tracing for block transactions that need enrichment (TraceRequest -> UrcEvent)
//! - Historical block healing for gap recovery
//! - Two specialized inspectors for different tracing scenarios

mod urci_tracer;
mod registry_config;
mod txpool_tracing;
mod block_tracing;
mod healing;

pub use urci_tracer::{TracerConfig, TracerMetrics, UrciTracer};
