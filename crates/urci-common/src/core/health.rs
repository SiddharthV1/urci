//! Health check and module status types

use chrono::{DateTime, Utc};
use derive_more::derive::{Deref, Display, From, Into};
use serde::{Deserialize, Serialize};

// Health check status following Commit-Boost pattern
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub healthy: bool,
    pub message: Option<String>,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Display, PartialEq, Eq, Hash, Deref, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UrciModule(pub String);

impl UrciModule {
    pub const EXEX: &'static str = "EXEX";
    pub const BLS_MERKLE_EXECUTOR: &'static str = "BLS_MERKLE_EXECUTOR";
    pub const TX_POOL_MONITOR: &'static str = "TX_POOL_MONITOR";
    pub const TRACER: &'static str = "TRACER";
    pub const ADAPTER: &'static str = "ADAPTER";
    pub const LABELER: &'static str = "LABELER";
}
