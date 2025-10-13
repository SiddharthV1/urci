use async_trait::async_trait;
use eyre::Result;
use sqlx::Transaction;
use urci_common::{
    core::UrciBlockUpdate, CollateralAdded, CollateralClaimed, OperatorOptedIn, OperatorOptedOut,
    OperatorRegistered, OperatorSlashed, OperatorUnregistered, UrciBatchedBlockRangeUpdate,
};

#[async_trait]
pub trait AdapterWriter: Send + Sync {
    /// Write a single block in its own transaction with advisory lock
    /// Returns the work_id of the written block for use as parent_work_id in next block
    async fn write_block(&mut self, block: UrciBlockUpdate) -> Result<alloy_primitives::B256>;

    /// Write multiple blocks in a SINGLE transaction with advisory lock (for proper parent_work_id FK handling)
    /// Returns the work_id of the LAST written block
    async fn write_blocks(
        &mut self,
        blocks: UrciBatchedBlockRangeUpdate,
    ) -> Result<alloy_primitives::B256>;

    // Specific event writing methods
    async fn write_registration_event(
        &mut self,
        owner: urci_common::Owner,
        event: OperatorRegistered,
        validation: urci_common::RegistrationValidationResult,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()>;

    async fn write_unregistration_event(
        &mut self,
        event: OperatorUnregistered,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()>;

    async fn write_opt_in_event(
        &mut self,
        event: OperatorOptedIn,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()>;

    async fn write_opt_out_event(
        &mut self,
        event: OperatorOptedOut,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()>;

    async fn write_collateral_claimed_event(
        &mut self,
        event: CollateralClaimed,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()>;

    async fn write_collateral_added_event(
        &mut self,
        event: CollateralAdded,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()>;

    async fn write_slashing_event(
        &mut self,
        event: OperatorSlashed,
        amount: urci_common::SlashAmountWei,
        call: urci_common::SlashingCall,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()>;

    /// Handle reorg by marking blocks non-canonical in the exact range from..=to
    /// Works on the exact range provided without assumptions
    /// Returns the work_id of the new canonical head after the reorg
    async fn handle_reorg(
        &mut self,
        from_block: u64,
        to_block: u64,
    ) -> Result<alloy_primitives::B256>;

    /// Finalize blocks up to the given block number
    /// Returns the work_id of the last finalized block
    async fn finalize_blocks(
        &mut self,
        chain_id: i64,
        up_to_block: i64,
    ) -> Result<alloy_primitives::B256>;

    async fn update_sync_status(
        &mut self,
        from_block: u64,
        to_block: u64,
        syncing: bool,
    ) -> Result<()>;
}
