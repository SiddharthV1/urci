//! Core cached executor implementation

use super::types::{CachedBLSValidation, CachedMerkleProof, CachedRegistrationValidation};
use crate::{BlsOps, MerkleOps};
use alloy_primitives::B256;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Wrapper that adds caching to any BlsOps + MerkleOps executor
/// Thouh the primary use case is for preprocessing registratoins via the txpool
pub struct CachedBlsExecutor<T>
where
    T: BlsOps + MerkleOps + Send + Sync,
{
    /// Inner executor that does actual work
    pub(super) inner: T,

    /// Cache for BLS signature validations
    pub(super) bls_cache: Arc<DashMap<B256, CachedBLSValidation>>,

    /// Cache for merkle proofs (optional - these are deterministic)
    pub(super) merkle_cache: Arc<DashMap<B256, CachedMerkleProof>>,

    /// Cache for complete registration validations
    pub(super) registration_cache: Arc<DashMap<B256, CachedRegistrationValidation>>,

    /// Cache configuration
    pub(super) ttl: Duration,
    pub(super) max_entries_per_cache: usize,
    pub(super) cache_merkle_proofs: bool,
}

impl<T> CachedBlsExecutor<T>
where
    T: BlsOps + MerkleOps + Send + Sync,
{
    /// Create a new cached executor wrapping an inner executor
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            bls_cache: Arc::new(DashMap::new()),
            merkle_cache: Arc::new(DashMap::new()),
            registration_cache: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(300), // 5 minutes default
            max_entries_per_cache: 10_000,
            cache_merkle_proofs: true, // Can disable if memory is a concern
        }
    }

    /// Configure cache settings
    pub fn with_config(mut self, ttl: Duration, max_size: usize, cache_merkle: bool) -> Self {
        self.ttl = ttl;
        self.max_entries_per_cache = max_size;
        self.cache_merkle_proofs = cache_merkle;
        self
    }

    /// Get the BLS validation cache for sharing
    pub fn bls_cache(&self) -> Arc<DashMap<B256, CachedBLSValidation>> {
        self.bls_cache.clone()
    }

    /// Get the fraud proof cache for sharing
    pub fn registration_cache(&self) -> Arc<DashMap<B256, CachedRegistrationValidation>> {
        self.registration_cache.clone()
    }

    /// Clean up expired entries from all caches
    pub fn cleanup(&self) {
        let now = Instant::now();

        // Clean BLS cache
        self.bls_cache
            .retain(|_, v| now.duration_since(v.timestamp) < self.ttl);
        self.enforce_size_limit(&self.bls_cache);

        // Clean merkle cache
        if self.cache_merkle_proofs {
            self.merkle_cache
                .retain(|_, v| now.duration_since(v.timestamp) < self.ttl);
            self.enforce_size_limit(&self.merkle_cache);
        }

        // Clean registration cache
        self.registration_cache
            .retain(|_, v| now.duration_since(v.timestamp) < self.ttl);
        self.enforce_size_limit(&self.registration_cache);
    }

    /// Enforce size limit on a cache
    pub(super) fn enforce_size_limit<V: Clone + std::fmt::Debug>(&self, cache: &DashMap<B256, V>) {
        if cache.len() > self.max_entries_per_cache {
            let to_remove = cache.len() - self.max_entries_per_cache;
            let keys: Vec<B256> = cache.iter().take(to_remove).map(|e| *e.key()).collect();
            for key in keys {
                cache.remove(&key);
            }
        }
    }
}
