use alloy_primitives::{hex, BlockHash, TxHash, B256};
use eyre::Result;
use sqlx::Transaction;
use tracing::{debug, error, info};
use urci_common::UrciBlockUpdate;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

// Helper methods for writing blocks, transactions, and events
impl PostgresAdapter {
    pub(super) async fn write_tx_events(
        &mut self,
        event: urci_common::UrciTxEvent,
        block_hash: BlockHash,
        block_number: i64,
        sqlx_tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        // Ensure sender address exists
        sqlx::query(sql_constants::INSERT_ADDRESS)
            .bind(event.tx_sender_address.as_slice())
            .bind(self.chain_id)
            .bind(Option::<bool>::None) // Need to check bytecode to determine if contract
            .execute(sqlx_tx.as_mut())
            .await?;

        // Insert transaction
        // Use sqlx::types::BigDecimal for NUMERIC columns
        use sqlx::types::BigDecimal;
        use std::str::FromStr;

        sqlx::query(sql_constants::INSERT_TRANSACTION_IGNORE)
            .bind(self.chain_id)
            .bind(event.transaction_hash.as_slice())
            .bind(block_hash.as_slice())
            .bind(event.tx_sender_address.as_slice())
            .bind(event.tx_to_address.map(|a| a.as_slice().to_vec()))
            .bind(BigDecimal::from_str(&event.tx_value.to_string()).unwrap_or_default())
            .bind(event.tx_gas as i64)
            .bind(BigDecimal::from_str(&event.tx_gas_price.to_string()).unwrap_or_default())
            .bind(event.tx_nonce as i64)
            .bind(event.transaction_input.map(|b| b.to_vec()))
            .bind(event.transaction_index as i32)
            .execute(sqlx_tx.as_mut())
            .await?;

        // Process each URC event
        for (log_index, urc_event) in event.urc_events.into_iter().enumerate() {
            self.write_event(
                urc_event,
                event.transaction_hash,
                block_hash,
                block_number,
                event.transaction_index as i32,
                log_index as i32,
                sqlx_tx,
            )
            .await?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn write_event(
        &mut self,
        event: urci_common::UrciEvent,
        tx_hash: TxHash,
        block_hash: BlockHash,
        block_number: i64,
        tx_index: i32,
        log_index: i32,
        sqlx_tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        use urci_common::UrciEventKind;

        // First, insert into events table to get event_id
        let event_type = match &event.event {
            UrciEventKind::Registration(_, _, _) => "OperatorRegistered",
            UrciEventKind::Unregistration(_) => "OperatorUnregistered",
            UrciEventKind::OptIn(_) => "OperatorOptedIn",
            UrciEventKind::OptOut(_) => "OperatorOptedOut",
            UrciEventKind::CollateralClamed(_) => "CollateralClaimed",
            UrciEventKind::CollateralAdded(_) => "CollateralAdded",
            UrciEventKind::Slashing(_, _, _) => "OperatorSlashed",
        };

        let event_id: i64 = sqlx::query_scalar(sql_constants::INSERT_EVENT_RETURNING_ID)
            .bind(self.chain_id)
            .bind(block_number)
            .bind(block_hash.as_slice())
            .bind(tx_hash.as_slice())
            .bind(tx_index)
            .bind(log_index)
            .bind(event_type)
            .bind(serde_json::to_value(&event)?)
            .bind(self.writer_id)
            .fetch_one(sqlx_tx.as_mut())
            .await?;

        // Now process the specific event with the event_id
        match event.event {
            UrciEventKind::Registration(owner, reg_event, validation) => {
                // Handle BLS validation errors gracefully - log and skip event
                match validation {
                    Ok(bls_data) => {
                        self.write_registration_event_internal(
                            owner, reg_event, bls_data, event_id, sqlx_tx,
                        )
                        .await
                    }
                    Err(e) => {
                        tracing::error!(
                            "BLS validation failed for registration event (owner={}, event_id={}): {}. Skipping event.",
                            owner, event_id, e
                        );
                        Ok(())
                    }
                }
            }
            UrciEventKind::Unregistration(unreg_event) => {
                self.write_unregistration_event_internal(unreg_event, event_id, sqlx_tx)
                    .await
            }
            UrciEventKind::OptIn(opt_in_event) => {
                self.write_opt_in_event_internal(opt_in_event, event_id, sqlx_tx)
                    .await
            }
            UrciEventKind::OptOut(opt_out_event) => {
                self.write_opt_out_event_internal(opt_out_event, event_id, sqlx_tx)
                    .await
            }
            UrciEventKind::CollateralClamed(claim_event) => {
                self.write_collateral_claimed_event_internal(claim_event, event_id, sqlx_tx)
                    .await
            }
            UrciEventKind::CollateralAdded(add_event) => {
                self.write_collateral_added_event_internal(add_event, event_id, sqlx_tx)
                    .await
            }
            UrciEventKind::Slashing(slash_event, amount, call) => {
                self.write_slashing_event_internal(slash_event, amount, *call, event_id, sqlx_tx)
                    .await
            }
        }
    }

    /// Write a single block within an existing transaction
    /// Returns the work_id of the written block
    pub(super) async fn write_block_internal(
        &mut self,
        block: &UrciBlockUpdate,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<B256> {
        let start = std::time::Instant::now();

        info!("Starting block write");

        debug!("Transaction started");

        // For root blocks (block number 1 with no parent), leave parent_work_id as NULL
        // The database constraint now enforces: block 0 MUST have NULL parent, all others MUST have non-NULL
        let parent_work_id = block.parent_work_id.map(|w| w.to_vec());

        debug!(
            parent_work_id = ?parent_work_id.as_ref().map(hex::encode),
            "Inserting block"
        );

        // Insert block FIRST (without active_event_id) so transactions can reference it
        // If parent_work_id is None, use INSERT_BLOCK_AS_ROOT to mark as root block
        let query_sql = if parent_work_id.is_none() {
            sql_constants::INSERT_BLOCK_AS_ROOT
        } else {
            sql_constants::INSERT_BLOCK_UPSERT
        };

        sqlx::query(query_sql)
            .bind(self.chain_id)
            .bind(block.block_number as i64)
            .bind(block.block_hash.as_slice())
            .bind(block.parent_block_hash.as_slice())
            .bind(parent_work_id)
            .bind(!block.reorged)
            .bind(false)
            .bind(block.timestamp as i64)
            .bind(None::<i64>) // active_event_id - will update later
            .bind(self.session_id.as_deref()) // session_id
            .execute(tx.as_mut())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to insert block");
                e
            })?;

        debug!("Block inserted successfully");

        // Now process transactions and events
        for (tx_idx, tx_events) in block.events.iter().enumerate() {
            debug!(
                tx_index = tx_idx,
                tx_hash = %tx_events.transaction_hash,
                urc_event_count = tx_events.urc_events.len(),
                "Processing transaction events"
            );

            self.write_tx_events(
                tx_events.clone(),
                block.block_hash,
                block.block_number as i64,
                tx,
            )
            .await
            .map_err(|e| {
                error!(
                    tx_index = tx_idx,
                    tx_hash = %tx_events.transaction_hash,
                    error = %e,
                    "Failed to write transaction events"
                );
                e
            })?;

            debug!(tx_index = tx_idx, "Transaction events written");
        }

        // Get the last event_id for this block to set as active_event_id
        debug!("Fetching last event ID");
        let last_event_id = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT MAX(id) FROM events WHERE block_hash = $1 AND chain_id = $2",
        )
        .bind(block.block_hash.as_slice())
        .bind(self.chain_id)
        .fetch_one(tx.as_mut())
        .await?;

        debug!(last_event_id = ?last_event_id, "Last event ID fetched");

        // Update block with active_event_id if we have events
        if let Some(event_id) = last_event_id {
            debug!(event_id = event_id, "Updating block with active_event_id");

            sqlx::query("UPDATE blocks SET active_event_id = $1 WHERE chain_id = $2 AND hash = $3")
                .bind(event_id)
                .bind(self.chain_id)
                .bind(block.block_hash.as_slice())
                .execute(tx.as_mut())
                .await?;

            // Update writer_status with last_indexed_event_id
            if let (Some(writer_id), Some(session_id)) = (self.writer_id, self.session_id.as_ref())
            {
                debug!(
                    writer_id = %writer_id,
                    session_id = %session_id,
                    "Updating writer status"
                );

                sqlx::query(sql_constants::INSERT_WRITER_STATUS_UPSERT)
                    .bind(writer_id)
                    .bind(session_id)
                    .bind(self.chain_id)
                    .bind(block.block_number as i64) // seen_block_from
                    .bind(block.block_number as i64) // seen_block_to
                    .bind(block.block_number as i64) // chain_tip
                    .bind(event_id) // last_indexed_event_id
                    .execute(tx.as_mut())
                    .await?;
            }
        }

        // Fetch the generated work_id for this block
        let work_id: Vec<u8> = sqlx::query_scalar(sql_constants::SELECT_BLOCK_WORK_ID)
            .bind(self.chain_id)
            .bind(block.block_number as i64)
            .fetch_one(tx.as_mut())
            .await?;

        // GAP HEALING: If we just wrote a block that orphaned children are waiting for,
        // update those children to link to this block's work_id
        // Example: Indexer started at block 1 (parent_work_id=NULL), now gap healing writes block 0
        // We need to UPDATE block 1 to set parent_work_id = block_0.work_id
        let update_result = sqlx::query(
            "UPDATE blocks
             SET parent_work_id = $1
             WHERE chain_id = $2
               AND parent_hash = $3
               AND parent_work_id IS NULL",
        )
        .bind(&work_id)
        .bind(self.chain_id)
        .bind(block.block_hash.as_slice())
        .execute(tx.as_mut())
        .await?;

        let updated_count = update_result.rows_affected();

        if updated_count > 0 {
            info!(
                updated_count = updated_count,
                block_number = block.block_number,
                block_hash = %hex::encode(block.block_hash),
                "Gap healing: Linked orphaned child blocks to newly discovered parent"
            );
        }

        let duration = start.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            work_id = %hex::encode(&work_id),
            "Block write completed (internal)"
        );

        Ok(B256::from_slice(&work_id))
    }
}

#[cfg(test)]
mod tests {
    use crate::postgres::adapter::PostgresAdapter;
    use crate::postgres::test_helpers::{create_test_block_with_event, setup_test_db};
    use crate::sql_constants;
    use crate::traits::{AdapterReader, AdapterWriter, Db};
    use alloy_primitives::{address, B256};
    use sqlx::Row;
    use urci_common::UrciBlockUpdate;

    // ============================================================================
    // GROUP 1: Basic Block Writing & Metadata
    // ============================================================================

    #[tokio::test]
    async fn test_root_block_constraints() {
        // Test that root block has is_root=TRUE and parent_work_id=NULL
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let start_height = 50u64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, start_height)
            .await
            .expect("Failed to initialize at block 50");

        // Write root block (deployment block)
        let block_50_hash = B256::from([50u8; 32]);
        let block_50 = UrciBlockUpdate {
            block_number: 50,
            block_hash: block_50_hash,
            parent_block_hash: B256::from([49u8; 32]),
            parent_work_id: None, // Root has NULL parent
            timestamp: 1000000,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block_50.clone())
            .await
            .expect("Failed to write root block");

        // VERIFY: Block 50 is marked as root and has NULL parent
        let pool = adapter.pool.clone();
        let (is_root, parent_work_id): (Option<bool>, Option<Vec<u8>>) = sqlx::query_as(
            "SELECT is_root, parent_work_id FROM blocks WHERE chain_id = $1 AND number = 50",
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch block 50");

        assert_eq!(
            is_root,
            Some(true),
            "Block 50 should be marked as is_root=TRUE"
        );
        assert!(
            parent_work_id.is_none(),
            "Root block must have parent_work_id=NULL"
        );

        // VERIFY: Only ONE block per chain can have NULL parent
        let null_parent_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM blocks WHERE chain_id = $1 AND parent_work_id IS NULL",
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count NULL parents");

        assert_eq!(
            null_parent_count, 1,
            "Only ONE block per chain can have NULL parent_work_id"
        );

        // Now write block 51 (non-root)
        let block_51_hash = B256::from([51u8; 32]);
        // Query database for block 50's work_id
        let work_50_bytes: Vec<u8> =
            sqlx::query_scalar("SELECT work_id FROM blocks WHERE chain_id = $1 AND hash = $2")
                .bind(chain_id)
                .bind(block_50_hash.as_slice())
                .fetch_one(&pool)
                .await
                .expect("Failed to get work_id for block 50");
        let work_50 = B256::from_slice(&work_50_bytes);

        let block_51 = UrciBlockUpdate {
            block_number: 51,
            block_hash: block_51_hash,
            parent_block_hash: block_50.block_hash,
            parent_work_id: Some(work_50),
            timestamp: 1000100,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block_51.clone())
            .await
            .expect("Failed to write block 51");

        // VERIFY: Block 51 is NOT root and has non-NULL parent
        let (is_root_51, parent_work_id_51): (Option<bool>, Option<Vec<u8>>) = sqlx::query_as(
            "SELECT is_root, parent_work_id FROM blocks WHERE chain_id = $1 AND number = 51",
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch block 51");

        assert_eq!(
            is_root_51,
            Some(false),
            "Block 51 should be marked as is_root=FALSE"
        );
        assert!(
            parent_work_id_51.is_some(),
            "Non-root block must have parent_work_id NOT NULL"
        );

        // VERIFY: Still only ONE block with NULL parent
        let null_parent_count_final: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM blocks WHERE chain_id = $1 AND parent_work_id IS NULL",
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count NULL parents");

        assert_eq!(
            null_parent_count_final, 1,
            "Still only ONE block can have NULL parent_work_id"
        );
    }

    #[tokio::test]
    async fn test_block_active_event_id_reflects_last_event() {
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

        // Write a block with events using the API
        let block_1 = create_test_block_with_event(1, None);
        let block_hash = block_1.block_hash; // Extract hash from created block
        adapter
            .write_block(block_1)
            .await
            .expect("Failed to write block 1");

        // Get all event IDs for this block to verify active_event_id points to the last one
        let event_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM events WHERE block_hash = $1 AND chain_id = $2 ORDER BY id",
        )
        .bind(block_hash.as_slice())
        .bind(chain_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch event IDs");

        assert!(
            !event_ids.is_empty(),
            "Block should have at least one event"
        );
        let last_event_id = *event_ids.last().unwrap();

        // Verify the block's active_event_id points to the last event
        let row =
            sqlx::query("SELECT active_event_id FROM blocks WHERE chain_id = $1 AND hash = $2")
                .bind(chain_id)
                .bind(block_hash.as_slice())
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch block");

        let active_event_id: Option<i64> = row.get("active_event_id");
        assert_eq!(
            active_event_id,
            Some(last_event_id),
            "active_event_id should point to the last event in the block"
        );
    }

    // ============================================================================
    // GROUP 2: Batch Writing & Auto-chaining
    // ============================================================================

    /// Test that sequential block writes correctly chain parent_work_id
    /// This replicates the bug where blocks arriving individually don't get their parent_work_id set
    #[tokio::test]
    async fn test_sequential_blocks_parent_work_id_chaining() {
        use urci_common::UrciBatchedBlockRangeUpdate;

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

        // Write block 1 (root block - no parent)
        let block_1 = UrciBlockUpdate {
            block_number: 1,
            block_hash: B256::from([1u8; 32]),
            parent_block_hash: B256::ZERO,
            timestamp: 1000000,
            work_id: None,
            parent_work_id: None, // Root block has no parent
            events: vec![],
            reorged: false,
            system_error: None,
        };

        let mut batch1 = UrciBatchedBlockRangeUpdate::new();
        batch1.add_block(block_1);
        let work_id_1 = adapter
            .write_blocks(batch1)
            .await
            .expect("Failed to write block 1");

        // Write blocks 2 and 3 in a BATCH to test auto-chaining
        // Only the first block in batch needs parent_work_id set
        // Block 2 should be auto-chained to block 1
        let block_2 = UrciBlockUpdate {
            block_number: 2,
            block_hash: B256::from([2u8; 32]),
            parent_block_hash: B256::from([1u8; 32]),
            timestamp: 1000002,
            work_id: None,
            parent_work_id: Some(work_id_1), // Only first block in batch needs parent_work_id
            events: vec![],
            reorged: false,
            system_error: None,
        };

        let block_3 = UrciBlockUpdate {
            block_number: 3,
            block_hash: B256::from([3u8; 32]),
            parent_block_hash: B256::from([2u8; 32]),
            timestamp: 1000003,
            work_id: None,
            parent_work_id: None, // Auto-chained by write_blocks_internal
            events: vec![],
            reorged: false,
            system_error: None,
        };

        let mut batch2 = UrciBatchedBlockRangeUpdate::new();
        batch2.add_block(block_2);
        batch2.add_block(block_3); // Add block 3 to same batch as block 2
        let _work_id_3 = adapter
            .write_blocks(batch2)
            .await
            .expect("Failed to write blocks 2-3 with auto-chaining");

        // Verify all three blocks exist with correct parent relationships
        let pool = adapter.pool.clone();
        let blocks: Vec<(i64, Vec<u8>, Option<Vec<u8>>)> = sqlx::query_as(
            "SELECT number, parent_hash, parent_work_id FROM blocks WHERE chain_id = $1 ORDER BY number"
        )
        .bind(chain_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch blocks");

        assert_eq!(blocks.len(), 3, "Should have 3 blocks");
        assert_eq!(blocks[0].0, 1, "First block should be number 1");
        assert_eq!(blocks[1].0, 2, "Second block should be number 2");
        assert_eq!(
            blocks[1].1,
            B256::from([1u8; 32]).as_slice(),
            "Block 2's parent_hash should be block 1's hash"
        );
        assert!(
            blocks[1].2.is_some(),
            "Block 2 should have parent_work_id set (first in batch)"
        );

        assert_eq!(blocks[2].0, 3, "Third block should be number 3");
        assert_eq!(
            blocks[2].1,
            B256::from([2u8; 32]).as_slice(),
            "Block 3's parent_hash should be block 2's hash"
        );
        assert!(
            blocks[2].2.is_some(),
            "Block 3 should have parent_work_id set (auto-chained in batch)"
        );
    }

    /// Test rejection of backfill attempts below start_height after chain has progressed
    /// Scenario: Indexer starts at block 1, successfully writes 1-3, then gap healing tries to backfill block 0
    /// Expected: Block 0 rejected, existing chain 1-3 remains intact
    #[tokio::test]
    async fn test_reject_backfill_below_start_height_with_existing_chain() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize at block 1");

        let pool = adapter.pool.clone();

        // PRODUCTION SCENARIO: Indexer starts at block 1, writes blocks 1-3
        // Block 0 exists on chain but wasn't seen yet

        let block_1 = UrciBlockUpdate {
            block_number: 1,
            block_hash: B256::from([1u8; 32]),
            parent_block_hash: B256::ZERO, // Points to block 0 which doesn't exist yet
            timestamp: 1000001,
            work_id: None,
            parent_work_id: None, // No parent in DB yet
            events: vec![],
            reorged: false,
            system_error: None,
        };

        let work_id_1 = adapter
            .write_block(block_1)
            .await
            .expect("Failed to write block 1");

        let block_2 = UrciBlockUpdate {
            block_number: 2,
            block_hash: B256::from([2u8; 32]),
            parent_block_hash: B256::from([1u8; 32]),
            timestamp: 1000002,
            work_id: None,
            parent_work_id: Some(work_id_1), // Points to block 1
            events: vec![],
            reorged: false,
            system_error: None,
        };

        let work_id_2 = adapter
            .write_block(block_2)
            .await
            .expect("Failed to write block 2");

        let block_3 = UrciBlockUpdate {
            block_number: 3,
            block_hash: B256::from([3u8; 32]),
            parent_block_hash: B256::from([2u8; 32]),
            timestamp: 1000003,
            work_id: None,
            parent_work_id: Some(work_id_2), // Points to block 2
            events: vec![],
            reorged: false,
            system_error: None,
        };

        adapter
            .write_block(block_3)
            .await
            .expect("Failed to write block 3");

        // NOW: Gap healing discovers block 0 is missing and tries to write it
        let block_0 = UrciBlockUpdate {
            block_number: 0,
            block_hash: B256::from([0u8; 32]),
            parent_block_hash: B256::ZERO,
            timestamp: 1000000,
            work_id: None,
            parent_work_id: None, // Starts as None
            events: vec![],
            reorged: false,
            system_error: None,
        };

        // EXPECTED: Block 0 should be REJECTED because it's below start_height (1)
        let result = adapter.write_block(block_0.clone()).await;
        assert!(
            result.is_err(),
            "Block 0 should be rejected (below start_height 1)"
        );

        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("below start_height")
                || error_msg.contains("0")
                || error_msg.contains("1"),
            "Error should mention block is below start_height, got: {}",
            error_msg
        );

        // VERIFY: Only 3 blocks exist (1, 2, 3). Block 0 was correctly rejected.
        let blocks: Vec<(i64, Option<Vec<u8>>, Option<bool>)> = sqlx::query_as(
            "SELECT number, parent_work_id, is_root FROM blocks WHERE chain_id = $1 ORDER BY number"
        )
        .bind(chain_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch blocks");

        assert_eq!(
            blocks.len(),
            3,
            "Should have 3 blocks (1, 2, 3). Block 0 was correctly rejected."
        );

        // Block 1 is the root (deployment block) - NULL parent
        assert_eq!(blocks[0].0, 1, "First block should be 1");
        assert!(blocks[0].1.is_none(), "Block 1 is root (NULL parent)");
        assert_eq!(
            blocks[0].2,
            Some(true),
            "Block 1 IS the root (indexer start point)"
        );

        // Block 2 has parent
        assert_eq!(blocks[1].0, 2, "Second block should be 2");
        assert!(
            blocks[1].1.is_some(),
            "Block 2 should have parent_work_id pointing to block 1"
        );

        // Block 3 has parent
        assert_eq!(blocks[2].0, 3, "Third block should be 3");
        assert!(
            blocks[2].1.is_some(),
            "Block 3 should have parent_work_id pointing to block 2"
        );
    }

    // ============================================================================
    // GROUP 3: Rejection Logic
    // ============================================================================

    #[tokio::test]
    async fn test_reject_block_immediately_on_first_write_below_start_height() {
        // Test basic rejection: first write attempt is below start_height
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        // SCENARIO: Indexer starts at block 100 (URC deployment block)
        let start_height = 100u64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");

        // Initialize with start_height = 100
        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, start_height)
            .await
            .expect("Failed to initialize at block 100");

        // Attempt to write block 50 (below start_height)
        let block_50 = UrciBlockUpdate {
            block_number: 50,
            block_hash: B256::from([50u8; 32]),
            parent_block_hash: B256::ZERO,
            timestamp: 1000000,
            work_id: None,
            parent_work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };

        // EXPECTED: Should be rejected
        let result = adapter.write_block(block_50).await;
        assert!(
            result.is_err(),
            "Block 50 should be rejected (below start_height 100)"
        );

        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("below start_height")
                || error_msg.contains("50")
                || error_msg.contains("100"),
            "Error should mention block is below start_height, got: {}",
            error_msg
        );
    }

    // ============================================================================
    // GROUP 4: Gap Healing
    // ============================================================================

    #[tokio::test]
    async fn test_orphaned_stream_block_gets_linked_when_gap_healed() {
        // Test gap healing UPDATE: orphaned stream block gets linked when missing parent is written
        // Scenario: Backfill reaches 102, stream writes orphaned 200, then gap healing writes 199
        // Expected: Block 200 automatically links to 199 via UPDATE query
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let start_height = 100u64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, start_height)
            .await
            .expect("Failed to initialize at block 100");

        // SCENARIO: Backfill reaches block 102, stream starts at block 200
        // Gap healing should fill blocks 103-200

        // Write root block 100 (deployment block)
        let block_100_hash = B256::from([100u8; 32]);
        let block_100 = UrciBlockUpdate {
            block_number: 100,
            block_hash: block_100_hash,
            parent_block_hash: B256::from([99u8; 32]),
            parent_work_id: None, // Root block
            timestamp: 1000000,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        let work_100 = adapter
            .write_block(block_100.clone())
            .await
            .expect("Failed to write block 100");

        // Write blocks 101-102 (backfill completed)
        let block_101_hash = B256::from([101u8; 32]);
        let block_101 = UrciBlockUpdate {
            block_number: 101,
            block_hash: block_101_hash,
            parent_block_hash: block_100_hash,
            parent_work_id: Some(work_100),
            timestamp: 1000100,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        let work_101 = adapter
            .write_block(block_101.clone())
            .await
            .expect("Failed to write block 101");

        let block_102_hash = B256::from([102u8; 32]);
        let block_102 = UrciBlockUpdate {
            block_number: 102,
            block_hash: block_102_hash,
            parent_block_hash: block_101_hash,
            parent_work_id: Some(work_101),
            timestamp: 1000200,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        let _work_102 = adapter
            .write_block(block_102.clone())
            .await
            .expect("Failed to write block 102");

        // Stream starts at block 200 (gap exists: 103-199)
        let block_200_hash = B256::from([200u8; 32]);
        let block_199_hash = B256::from([199u8; 32]); // Parent hash for block 200
        let block_200 = UrciBlockUpdate {
            block_number: 200,
            block_hash: block_200_hash,
            parent_block_hash: block_199_hash, // Parent not in DB yet
            parent_work_id: None,              // Orphaned (gap)
            timestamp: 1010000,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block_200.clone())
            .await
            .expect("Failed to write block 200");

        // Now gap healing discovers block 199
        let block_199 = UrciBlockUpdate {
            block_number: 199,
            block_hash: block_199_hash,
            parent_block_hash: B256::from([198u8; 32]), // Will be linked to 102 chain eventually
            parent_work_id: None,                       // Orphaned for now
            timestamp: 1009900,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        let work_199 = adapter
            .write_block(block_199.clone())
            .await
            .expect("Gap healing should write block 199");

        // VERIFY: Block 200 is now linked to block 199
        let pool = adapter.pool.clone();
        let block_200_parent: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT parent_work_id FROM blocks WHERE chain_id = $1 AND number = 200",
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch block 200");

        assert!(
            block_200_parent.is_some(),
            "Gap healing should have linked block 200 to block 199"
        );
        assert_eq!(
            B256::from_slice(&block_200_parent.unwrap()),
            work_199,
            "Block 200 should be linked to block 199's work_id"
        );

        // VERIFY: We have blocks 100-102 and 199-200 (gap remains 103-198)
        let block_count: i64 = sqlx::query_scalar(sql_constants::SELECT_COUNT_BLOCKS_BY_CHAIN)
            .bind(chain_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count blocks");

        assert_eq!(
            block_count, 5,
            "Should have 5 blocks (100, 101, 102, 199, 200)"
        );
    }

    #[tokio::test]
    async fn test_single_block_gap_healing_links_adjacent_orphan() {
        // Test minimal gap healing: orphaned block N+2 gets linked when single missing block N+1 is written
        // Simpler than backfill/stream scenario - just tests the core UPDATE mechanism
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let start_height = 10u64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, start_height)
            .await
            .expect("Failed to initialize at block 10");

        // Write root block 10
        let block_10_hash = B256::from([10u8; 32]);
        let block_10 = UrciBlockUpdate {
            block_number: 10,
            block_hash: block_10_hash,
            parent_block_hash: B256::from([9u8; 32]),
            parent_work_id: None, // Root
            timestamp: 1000000,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        let work_10 = adapter
            .write_block(block_10.clone())
            .await
            .expect("Failed to write block 10");

        // Stream writes block 12 (orphaned - parent 11 missing)
        let block_11_hash = B256::from([11u8; 32]); // This will be block 11's hash
        let block_12_hash = B256::from([12u8; 32]);
        let block_12 = UrciBlockUpdate {
            block_number: 12,
            block_hash: block_12_hash,
            parent_block_hash: block_11_hash, // References block 11
            parent_work_id: None,             // Orphaned (gap)
            timestamp: 1000200,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block_12.clone())
            .await
            .expect("Failed to write orphaned block 12");

        // VERIFY: Block 12 is orphaned (NULL parent_work_id)
        let pool = adapter.pool.clone();
        let block_12_parent_before: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT parent_work_id FROM blocks WHERE chain_id = $1 AND number = 12",
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch block 12");

        assert!(
            block_12_parent_before.is_none(),
            "Block 12 should be orphaned initially"
        );

        // Gap healing discovers block 11
        let block_11 = UrciBlockUpdate {
            block_number: 11,
            block_hash: block_11_hash, // Same hash that block 12 references
            parent_block_hash: block_10_hash,
            parent_work_id: Some(work_10),
            timestamp: 1000100,
            work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        let work_11 = adapter
            .write_block(block_11.clone())
            .await
            .expect("Gap healing should write block 11");

        // VERIFY: Block 12 is now linked to block 11 (gap healing UPDATE)
        let block_12_parent_after: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT parent_work_id FROM blocks WHERE chain_id = $1 AND number = 12",
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch block 12 after gap healing");

        assert!(
            block_12_parent_after.is_some(),
            "Gap healing should have linked block 12"
        );
        assert_eq!(
            B256::from_slice(&block_12_parent_after.unwrap()),
            work_11,
            "Block 12 should be linked to block 11's work_id"
        );

        // VERIFY: Complete chain 10→11→12
        let blocks: Vec<(i64, Option<Vec<u8>>)> = sqlx::query_as(
            "SELECT number, parent_work_id FROM blocks WHERE chain_id = $1 ORDER BY number",
        )
        .bind(chain_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch blocks");

        assert_eq!(blocks.len(), 3, "Should have 3 blocks (10, 11, 12)");
        assert_eq!(blocks[0].0, 10);
        assert!(blocks[0].1.is_none(), "Block 10 is root (NULL parent)");
        assert_eq!(blocks[1].0, 11);
        assert!(blocks[1].1.is_some(), "Block 11 has parent");
        assert_eq!(blocks[2].0, 12);
        assert!(blocks[2].1.is_some(), "Block 12 has parent (gap healed)");
    }

    // ============================================================================
    // GROUP 5: Fork and Reorg Handling
    // ============================================================================

    #[tokio::test]
    async fn test_block_fork_and_reorg() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");
        let registry_address = address!("0000000000000000000000000000000000000001");

        // Initialize database
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize database");

        // Create main chain blocks
        let mut last_work_id = None;

        // Block 1 (root)
        let block1 = UrciBlockUpdate {
            block_number: 1,
            block_hash: B256::from([1u8; 32]),
            parent_block_hash: B256::ZERO,
            timestamp: 1000001,
            work_id: None,
            parent_work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block1)
            .await
            .expect("Failed to write block 1");

        if let Some((_, _, work_id)) = adapter.get_block_by_number(1).await.unwrap() {
            last_work_id = Some(work_id);
        }

        // Block 2 (canonical)
        let block2 = UrciBlockUpdate {
            block_number: 2,
            block_hash: B256::from([2u8; 32]),
            parent_block_hash: B256::from([1u8; 32]),
            timestamp: 1000002,
            work_id: None,
            parent_work_id: last_work_id,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block2)
            .await
            .expect("Failed to write block 2");

        if let Some((_, _, work_id)) = adapter.get_block_by_number(2).await.unwrap() {
            last_work_id = Some(work_id);
        }

        // Block 3 (canonical)
        let block3 = UrciBlockUpdate {
            block_number: 3,
            block_hash: B256::from([3u8; 32]),
            parent_block_hash: B256::from([2u8; 32]),
            timestamp: 1000003,
            work_id: None,
            parent_work_id: last_work_id,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block3)
            .await
            .expect("Failed to write block 3");

        // Now create a fork - alternative block 3 with different hash
        // First we need to get block 2's work_id for the parent
        let (_, _, block2_work_id) = adapter.get_block_by_number(2).await.unwrap().unwrap();

        let block3_fork = UrciBlockUpdate {
            block_number: 3,
            block_hash: B256::from([33u8; 32]), // Different hash for forked block
            parent_block_hash: B256::from([2u8; 32]),
            timestamp: 1000003,
            work_id: None,
            parent_work_id: Some(block2_work_id),
            events: vec![],
            reorged: true, // Mark as reorged
            system_error: None,
        };

        // This should create a non-canonical block at height 3
        adapter
            .write_block(block3_fork)
            .await
            .expect("Failed to write forked block 3");

        // Verify fork state through the adapter
        let pool = adapter.pool();

        // Check canonical blocks
        let canonical_blocks =
            sqlx::query("SELECT number, hash FROM blocks WHERE canonical = true ORDER BY number")
                .fetch_all(pool)
                .await
                .expect("Failed to query canonical blocks");

        assert_eq!(canonical_blocks.len(), 3, "Should have 3 canonical blocks");

        // Check fork exists
        let fork_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blocks WHERE number = 3")
            .fetch_one(pool)
            .await
            .expect("Failed to count blocks at height 3");

        assert_eq!(
            fork_count, 2,
            "Should have 2 blocks at height 3 (canonical and fork)"
        );
    }

    #[tokio::test]
    async fn test_reorg_handling() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");
        let registry_address = address!("0000000000000000000000000000000000000001");

        // Initialize database
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize database");

        // Build a chain of blocks
        let mut last_work_id = None;
        for i in 1..=5 {
            let block = UrciBlockUpdate {
                block_number: i,
                block_hash: B256::from([i as u8; 32]),
                parent_block_hash: B256::from([(i.saturating_sub(1)) as u8; 32]),
                timestamp: 1000000 + i,
                work_id: None,
                parent_work_id: last_work_id,
                events: vec![],
                reorged: false,
                system_error: None,
            };
            adapter
                .write_block(block)
                .await
                .expect(&format!("Failed to write block {}", i));

            if let Some((_, _, work_id)) = adapter.get_block_by_number(i).await.unwrap() {
                last_work_id = Some(work_id);
            }
        }

        // Trigger reorg from block 3 to 5
        adapter
            .handle_reorg(3, 5)
            .await
            .expect("Failed to handle reorg");

        // Verify blocks 3-5 are marked as non-canonical
        let pool = adapter.pool();
        let non_canonical_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM blocks WHERE number >= 3 AND canonical = false",
        )
        .fetch_one(pool)
        .await
        .expect("Failed to count non-canonical blocks");

        assert_eq!(
            non_canonical_count, 3,
            "Blocks 3-5 should be non-canonical after reorg"
        );

        // Verify blocks 1-2 are still canonical
        let canonical_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM blocks WHERE number <= 2 AND canonical = true",
        )
        .fetch_one(pool)
        .await
        .expect("Failed to count canonical blocks");

        assert_eq!(canonical_count, 2, "Blocks 1-2 should remain canonical");
    }

    #[tokio::test]
    async fn test_multiple_forks() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");
        let registry_address = address!("0000000000000000000000000000000000000001");

        // Initialize database
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize database");

        // Create root block
        let block1 = UrciBlockUpdate {
            block_number: 1,
            block_hash: B256::from([1u8; 32]),
            parent_block_hash: B256::ZERO,
            timestamp: 1000001,
            work_id: None,
            parent_work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };
        adapter
            .write_block(block1)
            .await
            .expect("Failed to write block 1");

        let (_, _, block1_work_id) = adapter.get_block_by_number(1).await.unwrap().unwrap();

        // Create multiple competing blocks at height 2
        for i in 0..3 {
            let block2_variant = UrciBlockUpdate {
                block_number: 2,
                block_hash: B256::from([(20 + i) as u8; 32]),
                parent_block_hash: B256::from([1u8; 32]),
                timestamp: 1000002,
                work_id: None,
                parent_work_id: Some(block1_work_id),
                events: vec![],
                reorged: i > 0, // First one is canonical, others are forks
                system_error: None,
            };
            adapter
                .write_block(block2_variant)
                .await
                .expect(&format!("Failed to write block 2 variant {}", i));
        }

        // Verify we have multiple blocks at height 2
        let pool = adapter.pool();
        let block2_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blocks WHERE number = 2")
            .fetch_one(pool)
            .await
            .expect("Failed to count blocks at height 2");

        assert_eq!(
            block2_count, 3,
            "Should have 3 different blocks at height 2"
        );

        // Verify only one is canonical
        let canonical_at_2: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM blocks WHERE number = 2 AND canonical = true")
                .fetch_one(pool)
                .await
                .expect("Failed to count canonical blocks at height 2");

        assert_eq!(
            canonical_at_2, 1,
            "Only one block at height 2 should be canonical"
        );
    }

    #[tokio::test]
    async fn test_write_block_without_events() {
        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        // Create a block without events
        let block = UrciBlockUpdate {
            block_number: 1,
            block_hash: B256::from([1u8; 32]),
            parent_block_hash: B256::from([0u8; 32]),
            timestamp: 1000000,
            work_id: Some(B256::from([2u8; 32])),
            parent_work_id: None,
            events: vec![],
            reorged: false,
            system_error: None,
        };

        // Write the block
        adapter
            .write_block(block)
            .await
            .expect("Failed to write block");

        // Verify it was written
        use crate::traits::AdapterReader;
        let latest = adapter
            .get_latest_block()
            .await
            .expect("Failed to get latest block");
        assert!(latest.is_some());
        let (block_num, block_hash, work_id) = latest.unwrap();
        assert_eq!(block_num, 1);
        assert_eq!(block_hash, B256::from([1u8; 32]));
        // work_id is computed by the database, just verify it's not zero
        assert_ne!(work_id, B256::ZERO);
    }

    #[tokio::test]
    async fn test_write_block_with_events() {
        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        // Use TestFixture to create a block with actual URC registration event
        let block = create_test_block_with_event(1, None);
        let block_hash = block.block_hash;
        let tx_hash = block
            .events
            .first()
            .expect("Block should have tx")
            .transaction_hash;

        adapter
            .write_block(block)
            .await
            .expect("Failed to write block with events");

        // Verify block was written
        use crate::traits::AdapterReader;
        let latest = adapter
            .get_latest_block()
            .await
            .expect("Failed to get latest block");
        assert!(latest.is_some());
        let (block_num, returned_hash, _) = latest.unwrap();
        assert_eq!(block_num, 1);
        assert_eq!(returned_hash, block_hash);

        // Verify transaction was written
        let pool = adapter.pool.clone();
        let tx_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM transactions WHERE chain_id = $1 AND block_hash = $2",
        )
        .bind(chain_id)
        .bind(block_hash.as_slice())
        .fetch_one(&pool)
        .await
        .expect("Failed to count transactions");

        assert_eq!(tx_count, 1, "Should have one transaction");

        // Verify URC event was written
        let event_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE chain_id = $1 AND tx_hash = $2")
                .bind(chain_id)
                .bind(tx_hash.as_slice())
                .fetch_one(&pool)
                .await
                .expect("Failed to count events");

        assert_eq!(event_count, 1, "Should have one URC event");

        // Verify operator was created (registration event creates operator)
        let operator_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM operators WHERE chain_id = $1")
                .bind(chain_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to count operators");

        assert_eq!(
            operator_count, 1,
            "Should have one operator from registration event"
        );
    }
}
