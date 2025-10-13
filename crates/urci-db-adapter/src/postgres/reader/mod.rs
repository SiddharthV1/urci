use alloy_primitives::{Address, B256};
use async_trait::async_trait;
use eyre::Result;

use crate::traits::AdapterReader;
use urci_common::{OperatorState, WorkId};

use super::adapter::PostgresAdapter;

mod blocks;
mod collateral;
mod commitments;
mod operators;
mod slashing;

#[async_trait]
impl AdapterReader for PostgresAdapter {
    // ========================================================================
    // BLOCK METHODS
    // ========================================================================

    async fn get_latest_block(&self) -> Result<Option<(u64, B256, WorkId)>> {
        self.get_latest_block_impl().await
    }

    async fn get_latest_finalized_block(&self) -> Result<Option<(u64, B256, WorkId)>> {
        self.get_latest_finalized_block_impl().await
    }

    async fn get_block_by_number(&self, block_number: u64) -> Result<Option<(u64, B256, WorkId)>> {
        self.get_block_by_number_impl(block_number).await
    }

    async fn get_block_by_hash(&self, block_hash: &B256) -> Result<Option<(u64, B256, WorkId)>> {
        self.get_block_by_hash_impl(block_hash).await
    }

    async fn get_block_with_events(
        &self,
        chain_id: i64,
        block_number: i64,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.get_block_with_events_impl(chain_id, block_number)
            .await
    }

    async fn get_block_with_events_by_hash(
        &self,
        chain_id: i64,
        block_hash: &B256,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.get_block_with_events_by_hash_impl(chain_id, block_hash)
            .await
    }

    async fn get_block_head(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.get_block_head_impl(chain_id).await
    }

    async fn get_block_finalized(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.get_block_finalized_impl(chain_id).await
    }

    async fn get_chain_root(&self, chain_id: u64) -> Result<Option<(u64, WorkId)>> {
        self.get_chain_root_impl(chain_id).await
    }

    async fn get_fork_status(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::ForkStatusResponse>> {
        self.get_fork_status_impl(chain_id).await
    }

    async fn get_contract_config(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::ContractConfigResponse>> {
        self.get_contract_config_impl(chain_id).await
    }

    // ========================================================================
    // OPERATOR METHODS
    // ========================================================================

    async fn get_operators_at_block(&self, block_number: u64) -> Result<Vec<OperatorState>> {
        self.get_operators_at_block_impl(block_number).await
    }

    async fn get_operator_state(
        &self,
        operator: &Address,
        block_number: u64,
    ) -> Result<Option<OperatorState>> {
        self.get_operator_state_impl(operator, block_number).await
    }

    async fn get_slashable_operators(&self, block_number: u64) -> Result<Vec<OperatorState>> {
        self.get_slashable_operators_impl(block_number).await
    }

    async fn get_operator_by_registration_root(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorResponse>> {
        self.get_operator_by_registration_root_impl(chain_id, registration_root)
            .await
    }

    async fn get_operator_profile(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorResponse>> {
        self.get_operator_profile_impl(chain_id, registration_root)
            .await
    }

    async fn get_operator_by_owner(
        &self,
        chain_id: i64,
        owner_address: &Address,
    ) -> Result<Option<urci_common::api::OperatorResponse>> {
        self.get_operator_by_owner_impl(chain_id, owner_address)
            .await
    }

    async fn list_operators(
        &self,
        chain_id: i64,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Vec<urci_common::api::OperatorResponse>> {
        self.list_operators_impl(chain_id, limit, cursor).await
    }

    async fn search_operator_by_address(
        &self,
        chain_id: i64,
        address: &Address,
    ) -> Result<Option<urci_common::api::OperatorSearchResponse>> {
        self.search_operator_by_address_impl(chain_id, address)
            .await
    }

    async fn find_operators_by_pubkey(
        &self,
        chain_id: i64,
        pubkey_bytes: &[u8],
    ) -> Result<Vec<urci_common::api::OperatorWithPubkeyIndex>> {
        self.find_operators_by_pubkey_impl(chain_id, pubkey_bytes)
            .await
    }

    async fn get_operator_keys(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorKeysResponse>> {
        self.get_operator_keys_impl(chain_id, registration_root)
            .await
    }

    // ========================================================================
    // COLLATERAL METHODS
    // ========================================================================

    async fn get_operator_collateral(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorCollateralResponse>> {
        self.get_operator_collateral_impl(chain_id, registration_root)
            .await
    }

    async fn get_total_collateral(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::TotalCollateralResponse>> {
        self.get_total_collateral_impl(chain_id).await
    }

    async fn get_operators_by_collateral(
        &self,
        chain_id: i64,
        min_collateral: Option<String>,
        max_collateral: Option<String>,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorResponse>> {
        self.get_operators_by_collateral_impl(chain_id, min_collateral, max_collateral, limit)
            .await
    }

    async fn get_top_operators_by_collateral(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorResponse>> {
        self.get_top_operators_by_collateral_impl(chain_id, limit)
            .await
    }

    // ========================================================================
    // COMMITMENT METHODS (Slasher & Committer)
    // ========================================================================

    async fn get_operator_commitments(
        &self,
        chain_id: i64,
        registration_root: &B256,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Vec<urci_common::api::OperatorCommitmentResponse>> {
        self.get_operator_commitments_impl(chain_id, registration_root, limit, cursor)
            .await
    }

    async fn list_slashers(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::SlasherResponse>> {
        self.list_slashers_impl(chain_id, limit).await
    }

    async fn get_slasher_operators(
        &self,
        chain_id: i64,
        slasher_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorWithCommitmentResponse>> {
        self.get_slasher_operators_impl(chain_id, slasher_address, limit)
            .await
    }

    async fn get_slasher_stats(
        &self,
        chain_id: i64,
        slasher_address: &Address,
    ) -> Result<Option<urci_common::api::SlasherStatsResponse>> {
        self.get_slasher_stats_impl(chain_id, slasher_address).await
    }

    async fn list_committers(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::CommitterResponse>> {
        self.list_committers_impl(chain_id, limit).await
    }

    async fn get_committer_operators(
        &self,
        chain_id: i64,
        committer_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorWithCommitmentResponse>> {
        self.get_committer_operators_impl(chain_id, committer_address, limit)
            .await
    }

    // ========================================================================
    // SLASHING METHODS
    // ========================================================================

    async fn get_operator_slashing(
        &self,
        chain_id: i64,
        registration_root: &B256,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Option<urci_common::api::OperatorSlashingResponse>> {
        self.get_operator_slashing_impl(chain_id, registration_root, limit, cursor)
            .await
    }

    async fn get_slasher_slashing_events(
        &self,
        chain_id: i64,
        slasher_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::SlashingEventResponse>> {
        self.get_slasher_slashing_events_impl(chain_id, slasher_address, limit)
            .await
    }

    async fn get_committer_slashing_events(
        &self,
        chain_id: i64,
        committer_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::CommitterSlashingEventResponse>> {
        self.get_committer_slashing_events_impl(chain_id, committer_address, limit)
            .await
    }
}
