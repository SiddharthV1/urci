use alloy_primitives::{Address, B256};
use async_trait::async_trait;
use eyre::Result;
use urci_common::{
    api::{BlockResponse, OperatorCollateralResponse, OperatorCommitmentResponse, OperatorKeysResponse, OperatorResponse, OperatorSearchResponse, OperatorSlashingResponse, OperatorWithPubkeyIndex},
    core::{OperatorState, WorkId},
};

#[async_trait]
pub trait AdapterReader: Send + Sync {
    // ========================================================================
    // EXISTING METHODS (legacy - may need deprecation)
    // ========================================================================
    async fn get_latest_block(&self) -> Result<Option<(u64, B256, WorkId)>>;

    async fn get_latest_finalized_block(&self) -> Result<Option<(u64, B256, WorkId)>>;

    async fn get_block_by_number(&self, block_number: u64) -> Result<Option<(u64, B256, WorkId)>>;

    async fn get_block_by_hash(&self, block_hash: &B256) -> Result<Option<(u64, B256, WorkId)>>;

    async fn get_operators_at_block(&self, block_number: u64) -> Result<Vec<OperatorState>>;

    async fn get_operator_state(
        &self,
        operator: &Address,
        block_number: u64,
    ) -> Result<Option<OperatorState>>;

    async fn get_slashable_operators(&self, block_number: u64) -> Result<Vec<OperatorState>>;

    async fn get_chain_root(&self, chain_id: u64) -> Result<Option<(u64, WorkId)>>;

    // ========================================================================
    // NEW API METHODS (for urc-api)
    // ========================================================================

    /// Get block with all events by block number
    async fn get_block_with_events(
        &self,
        chain_id: i64,
        block_number: i64,
    ) -> Result<Option<BlockResponse>>;

    /// Get block with all events by block hash
    async fn get_block_with_events_by_hash(
        &self,
        chain_id: i64,
        block_hash: &B256,
    ) -> Result<Option<BlockResponse>>;

    /// Get current chain head block with events
    async fn get_block_head(&self, chain_id: i64) -> Result<Option<BlockResponse>>;

    /// Get last finalized block with events
    async fn get_block_finalized(&self, chain_id: i64) -> Result<Option<BlockResponse>>;

    // ========================================================================
    // OPERATOR METHODS (for urc-api)
    // ========================================================================

    /// Get operator by registration root
    async fn get_operator_by_registration_root(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<OperatorResponse>>;

    /// Get operator profile with commitments, slashing history, and timeline
    async fn get_operator_profile(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<OperatorResponse>>;

    /// Get operator by owner address
    async fn get_operator_by_owner(
        &self,
        chain_id: i64,
        owner_address: &Address,
    ) -> Result<Option<OperatorResponse>>;

    /// List operators with pagination
    async fn list_operators(
        &self,
        chain_id: i64,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Vec<OperatorResponse>>;

    /// Search for operator by any address (owner or in keys) and return with keys
    async fn search_operator_by_address(
        &self,
        chain_id: i64,
        address: &Address,
    ) -> Result<Option<OperatorSearchResponse>>;

    /// Find all operators that registered a specific BLS public key
    async fn find_operators_by_pubkey(
        &self,
        chain_id: i64,
        pubkey_bytes: &[u8],
    ) -> Result<Vec<OperatorWithPubkeyIndex>>;

    /// Get all BLS keys for an operator with merkle proofs
    async fn get_operator_keys(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<OperatorKeysResponse>>;

    /// Get operator collateral history
    async fn get_operator_collateral(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<OperatorCollateralResponse>>;

    /// Get operator commitments (opt-in/opt-out to slashers)
    async fn get_operator_commitments(
        &self,
        chain_id: i64,
        registration_root: &B256,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Vec<OperatorCommitmentResponse>>;

    /// Get operator slashing history
    async fn get_operator_slashing(
        &self,
        chain_id: i64,
        registration_root: &B256,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Option<OperatorSlashingResponse>>;

    // ========================================================================
    // FORK DETECTION METHODS
    // ========================================================================

    /// Get fork detection status for a chain
    async fn get_fork_status(&self, chain_id: i64) -> Result<Option<urci_common::api::ForkStatusResponse>>;

    // ========================================================================
    // COLLATERAL METHODS
    // ========================================================================

    /// Get total collateral statistics for a chain
    async fn get_total_collateral(&self, chain_id: i64) -> Result<Option<urci_common::api::TotalCollateralResponse>>;

    /// Get operators filtered by collateral range
    async fn get_operators_by_collateral(
        &self,
        chain_id: i64,
        min_collateral: Option<String>,
        max_collateral: Option<String>,
        limit: i64,
    ) -> Result<Vec<OperatorResponse>>;

    /// Get top operators by collateral
    async fn get_top_operators_by_collateral(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<OperatorResponse>>;

    // ========================================================================
    // SLASHER METHODS
    // ========================================================================

    /// List all slashers on a chain
    async fn list_slashers(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::SlasherResponse>>;

    /// Get operators committed to a slasher
    async fn get_slasher_operators(
        &self,
        chain_id: i64,
        slasher_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorWithCommitmentResponse>>;

    /// Get slasher statistics
    async fn get_slasher_stats(
        &self,
        chain_id: i64,
        slasher_address: &Address,
    ) -> Result<Option<urci_common::api::SlasherStatsResponse>>;

    /// Get slashing events for a slasher
    async fn get_slasher_slashing_events(
        &self,
        chain_id: i64,
        slasher_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::SlashingEventResponse>>;

    // ========================================================================
    // COMMITTER METHODS
    // ========================================================================

    /// List all committers on a chain
    async fn list_committers(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::CommitterResponse>>;

    /// Get operators using a committer
    async fn get_committer_operators(
        &self,
        chain_id: i64,
        committer_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorWithCommitmentResponse>>;

    /// Get slashing events initiated by a committer
    async fn get_committer_slashing_events(
        &self,
        chain_id: i64,
        committer_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::CommitterSlashingEventResponse>>;

    // ========================================================================
    // CONTRACT CONFIGURATION METHODS
    // ========================================================================

    /// Get deployed contract configuration for a chain
    async fn get_contract_config(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::ContractConfigResponse>>;
}