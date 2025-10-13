//! Writer module for PostgreSQL adapter
//!
//! This module is organized into submodules by functionality:
//! - `block_writer`: Core block, transaction, and event writing logic
//! - `error_handling`: Error classification and retry logic
//! - `registration`: Operator registration event processing
//! - `unregistration`: Operator unregistration event processing
//! - `opt_in`: Operator opt-in event processing
//! - `opt_out`: Operator opt-out event processing
//! - `collateral_claimed`: Collateral claim event processing
//! - `collateral_added`: Collateral addition event processing
//! - `slashing`: Slashing event processing
//! - `tests`: Unit and integration tests

use alloy_primitives::B256;
use async_trait::async_trait;
use eyre::Result;
use sqlx::Transaction;
use tracing::{error, instrument, warn};
use urci_common::UrciBlockUpdate;

use crate::sql_constants;
use crate::traits::AdapterWriter;

use super::adapter::PostgresAdapter;

mod block_writer;
mod collateral_added;
mod collateral_claimed;
mod error_handling;
mod opt_in;
mod opt_out;
mod registration;
mod slashing;
mod unregistration;

use error_handling::is_retryable_error;

// Helper methods for PostgresAdapter (not part of trait)
impl PostgresAdapter {
    async fn write_blocks_internal(
        &mut self,
        mut blocks: urci_common::UrciBatchedBlockRangeUpdate,
    ) -> Result<B256> {
        let mut tx = self.pool.begin().await?;

        // Take advisory lock for this chain to prevent concurrent writes
        self.acquire_tx_lock(&mut tx).await?;

        let mut last_work_id = B256::ZERO;

        // CRITICAL: Chain parent_work_id through the blocks in this batch
        // Block N+1 must have parent_work_id = Block N's work_id
        for i in 0..blocks.blocks.len() {
            // CRITICAL: Block 0 (genesis) MUST have parent_work_id = NULL
            if blocks.blocks[i].block_number == 0 {
                blocks.blocks[i].parent_work_id = None;
            } else if i > 0 {
                // If this is not the first block in the batch, set its parent_work_id to the previous block's work_id
                blocks.blocks[i].parent_work_id = Some(last_work_id);
            }

            last_work_id = self
                .write_block_internal(&blocks.blocks[i], &mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(last_work_id)
    }
}

#[async_trait]
impl AdapterWriter for PostgresAdapter {
    #[instrument(skip(self, block), fields(
        block_number = %block.block_number,
        block_hash = %block.block_hash,
        event_count = block.events.iter().map(|tx| tx.urc_events.len()).sum::<usize>(),
        reorged = block.reorged
    ))]
    async fn write_block(&mut self, mut block: UrciBlockUpdate) -> Result<B256> {
        // VALIDATION: Reject blocks below start_height (deployment block)
        // Blocks before URC deployment have no URC events and should not be indexed
        let start_height: Option<i64> =
            sqlx::query_scalar("SELECT start_height FROM chain_roots WHERE chain_id = $1")
                .bind(self.chain_id)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(start) = start_height {
            if (block.block_number as i64) < start {
                return Err(eyre::eyre!(
                    "Block {} is below start_height {} (URC deployment block). \
                     Blocks before deployment have no URC events and should not be indexed.",
                    block.block_number,
                    start
                ));
            }
        }

        // CRITICAL: Block 0 (genesis) MUST have parent_work_id = NULL
        if block.block_number == 0 {
            block.parent_work_id = None;
        }

        let mut tx = self.pool.begin().await?;

        // Take advisory lock for this chain to prevent concurrent writes
        self.acquire_tx_lock(&mut tx).await?;

        let work_id = self.write_block_internal(&block, &mut tx).await?;
        tx.commit().await?;
        Ok(work_id)
    }

    async fn write_blocks(
        &mut self,
        blocks: urci_common::UrciBatchedBlockRangeUpdate,
    ) -> Result<B256> {
        use std::sync::atomic::Ordering;

        // Check if we're in failed state - if so, write ALL blocks to error table
        if self.failed_state.load(Ordering::Relaxed) {
            error!(
                "Writer in failed state - writing {} blocks to error table",
                blocks.blocks.len()
            );

            for block in &blocks.blocks {
                // Best effort - log but don't fail if error table write fails
                if let Err(e) = self
                    .write_to_error_table(
                        block,
                        "Writer in failed state - all subsequent writes go to error table",
                        None,
                    )
                    .await
                {
                    error!(error = %e, "Failed to write to error table");
                }
            }

            // Return zero work_id to indicate failure
            return Ok(B256::ZERO);
        }

        // Try normal write
        let result = self.write_blocks_internal(blocks.clone()).await;

        match result {
            Ok(work_id) => Ok(work_id),
            Err(e) => {
                // Classify the error
                if is_retryable_error(&e) {
                    // Retryable error - propagate it so caller can retry
                    warn!(error = %e, "Retryable database error");
                    Err(e)
                } else {
                    // Data error - flip to failed state and write to error table
                    error!(error = %e, "NON-RETRYABLE database error - flipping to failed state");
                    self.failed_state.store(true, Ordering::Relaxed);

                    // Write all blocks in this batch to error table
                    for block in &blocks.blocks {
                        let error_context = serde_json::json!({
                            "error_type": format!("{:?}", e),
                            "error_message": e.to_string(),
                        });

                        if let Err(e2) = self
                            .write_to_error_table(block, &e.to_string(), Some(error_context))
                            .await
                        {
                            error!(error = %e2, "Failed to write to error table");
                        }
                    }

                    // Return zero work_id to indicate we handled it (don't crash)
                    Ok(B256::ZERO)
                }
            }
        }
    }

    async fn handle_reorg(&mut self, from_block: u64, to_block: u64) -> Result<B256> {
        let mut tx = self.pool.begin().await?;

        // Take advisory lock
        self.acquire_tx_lock(&mut tx).await?;

        // Mark the exact range from..=to as non-canonical
        sqlx::query(sql_constants::UPDATE_BLOCKS_CANONICAL_FALSE)
            .bind(from_block as i64)
            .bind(to_block as i64)
            .execute(&mut *tx)
            .await?;

        // Get the new canonical head work_id
        let head_work_id: Option<Vec<u8>> =
            sqlx::query_scalar(sql_constants::SELECT_CANONICAL_HEAD_WORK_ID)
                .bind(self.chain_id)
                .fetch_optional(&mut *tx)
                .await?;

        tx.commit().await?;

        Ok(head_work_id
            .map(|w| B256::from_slice(&w))
            .unwrap_or(B256::ZERO))
    }

    async fn finalize_blocks(&mut self, chain_id: i64, up_to_block: i64) -> Result<B256> {
        let mut tx = self.pool.begin().await?;

        // Take advisory lock
        self.acquire_tx_lock(&mut tx).await?;

        // Call the finalize_session_blocks function
        sqlx::query(sql_constants::CALL_FINALIZE_SESSION_BLOCKS)
            .bind(chain_id)
            .bind(up_to_block)
            .execute(&mut *tx)
            .await?;

        // Get the last finalized block's work_id
        let finalized_work_id: Option<Vec<u8>> =
            sqlx::query_scalar(sql_constants::SELECT_LAST_FINALIZED_WORK_ID)
                .bind(chain_id)
                .fetch_optional(&mut *tx)
                .await?;

        tx.commit().await?;

        Ok(finalized_work_id
            .map(|w| B256::from_slice(&w))
            .unwrap_or(B256::ZERO))
    }

    async fn update_sync_status(
        &mut self,
        from_block: u64,
        to_block: u64,
        _syncing: bool,
    ) -> Result<()> {
        if let (Some(writer_id), Some(session_id)) = (self.writer_id, self.session_id.as_ref()) {
            let chain_tip: Option<i64> =
                sqlx::query_scalar(sql_constants::SELECT_MAX_CANONICAL_BLOCK)
                    .fetch_optional(&self.pool)
                    .await?;

            sqlx::query(sql_constants::INSERT_WRITER_STATUS_UPSERT)
                .bind(writer_id)
                .bind(session_id)
                .bind(self.chain_id)
                .bind(from_block as i64)
                .bind(to_block as i64)
                .bind(chain_tip.unwrap_or(to_block as i64))
                .bind(None::<i64>) // last_indexed_event_id - not tied to specific event
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn write_registration_event(
        &mut self,
        owner: urci_common::Owner,
        event: urci_common::OperatorRegistered,
        validation: urci_common::RegistrationValidationResult,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        self.write_registration_event_internal(owner, event, validation, event_id, tx)
            .await
    }

    async fn write_unregistration_event(
        &mut self,
        event: urci_common::OperatorUnregistered,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        self.write_unregistration_event_internal(event, event_id, tx)
            .await
    }

    async fn write_opt_in_event(
        &mut self,
        event: urci_common::OperatorOptedIn,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        self.write_opt_in_event_internal(event, event_id, tx).await
    }

    async fn write_opt_out_event(
        &mut self,
        event: urci_common::OperatorOptedOut,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        self.write_opt_out_event_internal(event, event_id, tx).await
    }

    async fn write_collateral_claimed_event(
        &mut self,
        event: urci_common::CollateralClaimed,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        self.write_collateral_claimed_event_internal(event, event_id, tx)
            .await
    }

    async fn write_collateral_added_event(
        &mut self,
        event: urci_common::CollateralAdded,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        self.write_collateral_added_event_internal(event, event_id, tx)
            .await
    }

    async fn write_slashing_event(
        &mut self,
        event: urci_common::OperatorSlashed,
        amount: urci_common::SlashAmountWei,
        call: urci_common::SlashingCall,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        self.write_slashing_event_internal(event, amount, call, event_id, tx)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::postgres::adapter::PostgresAdapter;
    use crate::postgres::test_helpers::{create_test_block_with_event, setup_test_db};
    use crate::sql_constants;
    use crate::traits::{AdapterWriter, Db};
    use alloy_primitives::{address, B256};
    use urci_common::UrciBlockUpdate;

    #[tokio::test]
    async fn test_writer_status_tracks_last_indexed_event() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        let pool = adapter.pool.clone();

        // Create first block with an event using write_block API
        let block_1 = create_test_block_with_event(1, None);
        adapter
            .write_block(block_1)
            .await
            .expect("Failed to write block 1");

        // Verify writer_status was updated with last_indexed_event_id
        let writer_id = adapter.writer_id.expect("Writer ID should be set");
        let session_id = adapter
            .session_id
            .clone()
            .expect("Session ID should be set");

        let row = sqlx::query(
            "SELECT last_indexed_event_id FROM writer_status
             WHERE writer_id = $1 AND session_id = $2 AND chain_id = $3",
        )
        .bind(writer_id)
        .bind(&session_id)
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch writer_status");

        let last_event_1: Option<i64> = sqlx::Row::get(&row, "last_indexed_event_id");
        assert!(
            last_event_1.is_some(),
            "writer_status should track the last indexed event_id"
        );

        // Get the work_id from block 1 to use as parent for block 2
        let parent_work_id: Vec<u8> = sqlx::query_scalar(sql_constants::SELECT_BLOCK_WORK_ID)
            .bind(chain_id)
            .bind(1i64)
            .fetch_one(&pool)
            .await
            .expect("Failed to get block 1 work_id");

        // Convert Vec<u8> to B256
        let parent_work_id_b256 = B256::from_slice(&parent_work_id);

        // Create second block with an event
        let block_2 = create_test_block_with_event(2, Some(parent_work_id_b256));
        adapter
            .write_block(block_2)
            .await
            .expect("Failed to write block 2");

        // Verify writer_status now has the second event_id
        let row2 = sqlx::query(
            "SELECT last_indexed_event_id FROM writer_status
             WHERE writer_id = $1 AND session_id = $2 AND chain_id = $3",
        )
        .bind(writer_id)
        .bind(&session_id)
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch writer_status after second event");

        let last_event_2: Option<i64> = sqlx::Row::get(&row2, "last_indexed_event_id");
        assert!(
            last_event_2.is_some(),
            "writer_status should have event_id after block 2"
        );
        assert!(
            last_event_2.unwrap() > last_event_1.unwrap(),
            "writer_status should update to track the latest indexed event_id"
        );
    }

    #[tokio::test]
    async fn test_last_finalized_block_view_by_chain() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        let pool = adapter.pool.clone();

        // Insert multiple blocks, tracking parent_work_id
        let total_blocks = 5;
        let finalization_threshold = 3; // Blocks 1-3 are finalized

        let mut parent_work_id: Option<Vec<u8>> = None;
        for i in 1..=total_blocks {
            let block_hash = B256::from([i as u8; 32]);
            let parent_hash = if i == 1 {
                B256::ZERO
            } else {
                B256::from([(i - 1) as u8; 32])
            };

            let work_id: Vec<u8> =
                sqlx::query_scalar(sql_constants::INSERT_BLOCK_RETURNING_WORK_ID)
                    .bind(chain_id)
                    .bind(i as i64)
                    .bind(block_hash.as_slice())
                    .bind(parent_hash.as_slice())
                    .bind(&parent_work_id)
                    .bind(1000000i64 + i)
                    .bind(true) // canonical
                    .bind(i <= finalization_threshold) // blocks 1-3 are finalized
                    .bind(None::<&str>) // session_id (NULL for test)
                    .fetch_one(&pool)
                    .await
                    .expect("Failed to insert block");

            parent_work_id = Some(work_id);
        }

        // Query the view for last finalized block for this chain
        let row = sqlx::query(
            "SELECT number, hash FROM view_last_finalized_block_by_chain WHERE chain_id = $1",
        )
        .bind(chain_id)
        .fetch_optional(&pool)
        .await
        .expect("Failed to query view_last_finalized_block_by_chain");

        assert!(
            row.is_some(),
            "View should return a row for chain with finalized blocks"
        );
        let row = row.unwrap();
        let number: i64 = sqlx::Row::get(&row, "number");
        let hash: Vec<u8> = sqlx::Row::get(&row, "hash");

        assert_eq!(
            number, finalization_threshold as i64,
            "Last finalized block should be block {}",
            finalization_threshold
        );
        assert_eq!(
            hash,
            B256::from([finalization_threshold as u8; 32]).as_slice(),
            "Hash should match block {}",
            finalization_threshold
        );
    }

    #[tokio::test]
    async fn test_finalizer_deletes_empty_finalized_blocks() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        let pool = adapter.pool.clone();

        // Write block 1 with events
        let block_1 = create_test_block_with_event(1, None);
        adapter
            .write_block(block_1)
            .await
            .expect("Failed to write block 1");

        // Get work_id from block 1 for block 2
        let work_id_1: Vec<u8> = sqlx::query_scalar(sql_constants::SELECT_BLOCK_WORK_ID)
            .bind(chain_id)
            .bind(1i64)
            .fetch_one(&pool)
            .await
            .expect("Failed to get block 1 work_id");

        // Write block 2 WITHOUT events (empty block) using UrciBlockUpdate
        let block_2 = UrciBlockUpdate {
            block_number: 2,
            block_hash: B256::from([2u8; 32]),
            parent_block_hash: B256::from([1u8; 32]),
            timestamp: 1000002,
            work_id: None,
            parent_work_id: Some(B256::from_slice(&work_id_1)),
            events: vec![], // NO EVENTS
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block_2)
            .await
            .expect("Failed to write block 2");

        // Get work_id from block 2 for block 3
        let work_id_2: Vec<u8> = sqlx::query_scalar(sql_constants::SELECT_BLOCK_WORK_ID)
            .bind(chain_id)
            .bind(2i64)
            .fetch_one(&pool)
            .await
            .expect("Failed to get block 2 work_id");

        // Write block 3 with events
        let block_3 = create_test_block_with_event(3, Some(B256::from_slice(&work_id_2)));
        adapter
            .write_block(block_3)
            .await
            .expect("Failed to write block 3");

        // Verify all 3 blocks exist before finalization
        let count_before: i64 = sqlx::query_scalar(sql_constants::SELECT_COUNT_BLOCKS_BY_CHAIN)
            .bind(chain_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count blocks");
        assert_eq!(
            count_before, 3,
            "Should have 3 blocks before finalizer runs"
        );

        // Finalize up to block 3
        let finalize_up_to = 3i64;
        sqlx::query(sql_constants::CALL_FINALIZE_SESSION_BLOCKS)
            .bind(chain_id)
            .bind(finalize_up_to)
            .execute(&pool)
            .await
            .expect("Failed to run finalizer function");

        // Verify block 2 (empty block) was deleted, blocks 1 and 3 remain
        let remaining_blocks: Vec<i64> =
            sqlx::query_scalar("SELECT number FROM blocks WHERE chain_id = $1 ORDER BY number")
                .bind(chain_id)
                .fetch_all(&pool)
                .await
                .expect("Failed to fetch remaining blocks");

        // Expected: blocks with events (1 and 3) remain, empty block (2) deleted
        let expected_remaining = vec![1i64, finalize_up_to];
        assert_eq!(
            remaining_blocks, expected_remaining,
            "Block 2 (empty block) should be deleted, blocks 1 and 3 should remain"
        );

        // Verify blocks 1 and 3 are marked as finalized
        let finalized_blocks: Vec<i64> = sqlx::query_scalar(
            "SELECT number FROM blocks WHERE chain_id = $1 AND finalized = true ORDER BY number",
        )
        .bind(chain_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch finalized blocks");

        assert_eq!(
            finalized_blocks, expected_remaining,
            "Blocks 1 and 3 should be marked as finalized"
        );
    }

    #[tokio::test]
    async fn test_fork_view_tracks_active_event_id() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        // Create writer 1
        let mut adapter1 = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect adapter1");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter1
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        let pool = adapter1.pool.clone();

        // Writer 1 writes blocks 1-9 to build a chain
        let mut last_work_id = None;
        for block_num in 1..=9 {
            let block = create_test_block_with_event(block_num, last_work_id);
            adapter1
                .write_block(block)
                .await
                .expect(&format!("Failed to write block {}", block_num));

            // Get work_id for next block
            last_work_id = Some(B256::from_slice(
                &sqlx::query_scalar::<_, Vec<u8>>(sql_constants::SELECT_BLOCK_WORK_ID)
                    .bind(chain_id)
                    .bind(block_num as i64)
                    .fetch_one(&pool)
                    .await
                    .expect(&format!("Failed to get work_id for block {}", block_num)),
            ));
        }

        // Writer 1 writes block 10 with event A
        let block_10_a = create_test_block_with_event(10, last_work_id);
        adapter1
            .write_block(block_10_a)
            .await
            .expect("Failed to write block 10 from writer 1");

        // Update writer_status to mark writer 1 as synced (seen_block_to = chain_tip = 10)
        if let (Some(writer_id), Some(session_id)) =
            (adapter1.writer_id, adapter1.session_id.as_ref())
        {
            sqlx::query(
                "UPDATE writer_status SET seen_block_to = 10, chain_tip = 10, updated_at = NOW()
                 WHERE writer_id = $1 AND session_id = $2 AND chain_id = $3",
            )
            .bind(writer_id)
            .bind(session_id)
            .bind(chain_id)
            .execute(&pool)
            .await
            .expect("Failed to update writer 1 status");
        }

        // Create writer 2 (simulating a fork scenario - different view of block 10)
        let mut adapter2 = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect adapter2");

        adapter2
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize adapter2");

        // Writer 2 writes blocks 1-9 (same as writer 1)
        let mut last_work_id_2 = None;
        for block_num in 1..=9 {
            // Reuse existing blocks by just updating writer_status
            last_work_id_2 = Some(B256::from_slice(
                &sqlx::query_scalar::<_, Vec<u8>>(sql_constants::SELECT_BLOCK_WORK_ID)
                    .bind(chain_id)
                    .bind(block_num as i64)
                    .fetch_one(&pool)
                    .await
                    .expect(&format!("Failed to get work_id for block {}", block_num)),
            ));
        }

        // Writer 2 writes a DIFFERENT block 10 with event B (fork!)
        let mut block_10_b = create_test_block_with_event(110, last_work_id_2); // Use different event data
        block_10_b.block_number = 10; // But at same height
        block_10_b.block_hash = B256::from([110u8; 32]); // Different hash
        adapter2
            .write_block(block_10_b)
            .await
            .expect("Failed to write block 10 from writer 2");

        // Update writer_status to mark writer 2 as synced (seen_block_to = chain_tip = 10)
        if let (Some(writer_id), Some(session_id)) =
            (adapter2.writer_id, adapter2.session_id.as_ref())
        {
            sqlx::query(
                "UPDATE writer_status SET seen_block_to = 10, chain_tip = 10, updated_at = NOW()
                 WHERE writer_id = $1 AND session_id = $2 AND chain_id = $3",
            )
            .bind(writer_id)
            .bind(session_id)
            .bind(chain_id)
            .execute(&pool)
            .await
            .expect("Failed to update writer 2 status");
        }

        // Query the fork view - it should show both blocks at height 10 with different active_event_ids
        let rows = sqlx::query(
            "SELECT active_event_id, hash FROM view_fork_head_events
             WHERE chain_id = $1
             ORDER BY active_event_id",
        )
        .bind(chain_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to query view_fork_head_events");

        // Should see 2 different head events at the highest common synced block (block 10)
        assert_eq!(
            rows.len(),
            2,
            "Should see 2 different head events at block 10 (fork detected)"
        );

        let event_ids: Vec<Option<i64>> = rows
            .iter()
            .map(|r| sqlx::Row::get(r, "active_event_id"))
            .collect();
        assert!(
            event_ids[0] != event_ids[1],
            "The two head events should have different active_event_ids indicating a fork"
        );
    }
}
