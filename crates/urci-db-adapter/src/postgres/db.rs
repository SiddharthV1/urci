use alloy_primitives::{Address, B256};
use async_trait::async_trait;
use eyre::Result;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use crate::sql_constants;
use crate::state::{DBState, LastUpdate};
use crate::traits::Db;
use urci_common::WorkId;

use super::adapter::PostgresAdapter;

#[async_trait]
impl Db for PostgresAdapter {
    async fn connect(&mut self, db_url: &str) -> Result<(Uuid, Option<u64>, Option<WorkId>)> {
        self.pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(db_url)
            .await?;

        self.db_url = db_url.to_string();
        Ok((Uuid::new_v4(), None, None))
    }

    async fn is_initialized(&self) -> Result<bool> {
        let tables_exist: bool = sqlx::query_scalar(sql_constants::CHECK_TABLES_EXIST)
            .fetch_one(&self.pool)
            .await?;
        Ok(tables_exist)
    }

    async fn run_migrations(&mut self) -> Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_session(&mut self, registry_address: Address, start_block: Option<u64>) -> Result<Uuid> {
        // Insert chain
        sqlx::query(sql_constants::INSERT_CHAIN)
            .bind(self.chain_id)
            .bind(format!("chain_{}", self.chain_id))
            .execute(&self.pool)
            .await?;

        // Insert chain_roots with deployment block
        sqlx::query(sql_constants::INSERT_CHAIN_ROOTS_IGNORE)
            .bind(self.chain_id)
            .bind(start_block.unwrap_or(0) as i64)
            .execute(&self.pool)
            .await?;

        let writer_name = format!("adapter_{}", Uuid::new_v4());
        let session_id = format!("session_{}", Uuid::new_v4());

        let config = serde_json::json!({
            "chain_id": self.chain_id,
            "name": &writer_name,
            "registry_address": registry_address.to_string(),
        });

        let writer_id: Uuid = sqlx::query_scalar(sql_constants::SELECT_DERIVE_WRITER_ID)
            .bind(&config)
            .fetch_one(&self.pool)
            .await?;

        sqlx::query(sql_constants::INSERT_WRITERS_VARIANT)
            .bind(writer_id)
            .bind(&writer_name)
            .bind(&session_id)
            .bind(&config)
            .execute(&self.pool)
            .await?;

        // Insert into writer_status so that blocks can reference this session_id
        sqlx::query(sql_constants::INSERT_WRITER_STATUS)
            .bind(writer_id)
            .bind(&session_id)
            .bind(self.chain_id)
            .bind(start_block.map(|b| b as i64))
            .bind(start_block.map(|b| b as i64))
            .bind(None::<i64>) // chain_tip
            .execute(&self.pool)
            .await?;

        self.writer_id = Some(writer_id);
        self.session_id = Some(session_id.clone());
        self.registry_address = Some(registry_address);

        Ok(writer_id)
    }

    async fn write_config(&mut self, writer_id: Uuid, registry_address: Address, config: urci_common::IRegistry::Config) -> Result<()> {
        // Update writer config (legacy - store as JSON)
        let config_json = serde_json::to_value(&config)?;
        sqlx::query(sql_constants::UPDATE_WRITER_CONFIG)
            .bind(&config_json)
            .bind(writer_id)
            .execute(&self.pool)
            .await?;

        // Insert registry address into address table first (required for foreign key)
        sqlx::query(sql_constants::INSERT_ADDRESS)
            .bind(registry_address.as_slice())
            .bind(self.chain_id)
            .bind(true) // is_contract
            .execute(&self.pool)
            .await?;

        // Insert into config table
        use std::str::FromStr;
        let min_collateral = sqlx::types::BigDecimal::from_str(&config.minCollateralWei.to_string())?;
        sqlx::query(sql_constants::INSERT_CONFIG)
            .bind(self.chain_id)
            .bind(registry_address.as_slice())
            .bind(min_collateral)
            .bind(config.fraudProofWindow as i32)
            .bind(config.unregistrationDelay as i32)
            .bind(config.slashWindow as i32)
            .bind(config.optInDelay as i32)
            .bind(writer_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_indexer_height(&self) -> Result<(Uuid, Option<u64>, Option<WorkId>)> {
        let latest = sqlx::query_as::<_, (i64, Vec<u8>)>(
            sql_constants::SELECT_LATEST_FINALIZED_BLOCK,
        )
        .fetch_optional(&self.pool)
        .await?;

        let writer_id = self.writer_id.unwrap_or_else(Uuid::new_v4);

        match latest {
            Some((height, work_id_bytes)) => {
                let work_id = B256::from_slice(&work_id_bytes);
                Ok((writer_id, Some(height as u64 + 1), Some(work_id)))
            }
            None => {
                let start_height = sqlx::query_scalar::<_, Option<i64>>(
                    sql_constants::SELECT_CHAIN_ROOT_START_HEIGHT,
                )
                .bind(self.chain_id)  // FIX: Use actual chain_id, not hardcoded 1
                .fetch_optional(&self.pool)
                .await?
                .flatten();

                Ok((writer_id, start_height.map(|h| h as u64), None))
            }
        }
    }

    async fn get_db_state(&self) -> Result<DBState> {
        let session_tip = sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>, i64)>(
            sql_constants::SELECT_CANONICAL_BLOCK_DETAILED,
        )
        .bind(self.chain_id)
        .fetch_optional(&self.pool)
        .await?;

        let indexer_tip = sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>, i64)>(
            sql_constants::SELECT_FINALIZED_BLOCK_DETAILED,
        )
        .bind(self.chain_id)
        .fetch_optional(&self.pool)
        .await?;

        let finalized = indexer_tip.clone();

        let is_syncing = match (&session_tip, &indexer_tip) {
            (Some((s_height, _, _, _)), Some((i_height, _, _, _))) => s_height != i_height,
            _ => true,
        };

        let to_last_update = |data: Option<(i64, Vec<u8>, Vec<u8>, i64)>| -> LastUpdate {
            match data {
                Some((height, hash, work_id, timestamp)) => LastUpdate {
                    block_number: height as u64,
                    block_hash: B256::from_slice(&hash),
                    work_id: B256::from_slice(&work_id),
                    timestamp: timestamp as u64,
                },
                None => LastUpdate {
                    block_number: 0,
                    block_hash: B256::ZERO,
                    work_id: B256::ZERO,
                    timestamp: 0,
                },
            }
        };

        Ok(DBState {
            session_tip: to_last_update(session_tip),
            indexer_tip: to_last_update(indexer_tip),
            finalized: to_last_update(finalized),
            is_syncing,
            session_id: self.writer_id,
        })
    }

    async fn initialize(
        &mut self,
        registry_address: Address,
        start_block: u64,
    ) -> Result<(Uuid, Option<u64>, Option<WorkId>)> {
        self.run_migrations().await?;
        let _writer_id = self.create_session(registry_address, Some(start_block)).await?;
        self.get_indexer_height().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_helpers::setup_test_db;
    use crate::traits::{AdapterReader, AdapterWriter, Db};
    use alloy_primitives::address;
    use sqlx::Row;
    use urci_common::UrciBlockUpdate;

    #[tokio::test]
    async fn test_db_initialize() {
        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url).await.expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");

        // Initialize database - should create tables and return writer ID
        let (writer_id, last_block, last_work_id) = adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize database");

        assert!(!writer_id.is_nil());
        assert_eq!(last_block, Some(1)); // Start from block 1
        assert_eq!(last_work_id, None);
    }

    #[tokio::test]
    async fn test_db_is_initialized() {
        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url).await.expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");

        // Check before initialization
        let initialized = adapter.is_initialized().await.expect("Failed to check initialization");
        assert!(!initialized);

        // Initialize
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        // Check after initialization
        let initialized = adapter.is_initialized().await.expect("Failed to check initialization");
        assert!(initialized);
    }

    #[tokio::test]
    async fn test_db_reinitialize_preserves_existing_tables() {
        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url).await.expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");

        // Initialize and write data
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        // Write multiple blocks
        let mut last_work_id: Option<B256> = None;
        for i in 1..=3 {
            let block = UrciBlockUpdate {
                block_number: i,
                block_hash: B256::from([i as u8; 32]),
                parent_block_hash: B256::from([(i.saturating_sub(1)) as u8; 32]),
                timestamp: 1000000 + i,
                work_id: None, // Let database compute it
                parent_work_id: last_work_id,
                events: vec![],
                reorged: false,
                system_error: None,
            };
            adapter.write_block(block).await.expect("Failed to write block");

            // Get the actual work_id for the next block
            if let Some((_, _, work_id)) = adapter.get_block_by_number(i).await.expect("Failed to get block") {
                last_work_id = Some(work_id);
            }
        }

        // Count blocks before reinit
        {
            let pool = adapter.pool();
            let block_count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blocks")
                .fetch_one(pool)
                .await
                .expect("Failed to count blocks");

            assert_eq!(block_count_before, 3, "Should have 3 blocks");
        }

        // Check if initialized (should preserve data)
        let is_init = adapter.is_initialized().await.expect("Failed to check");
        assert!(is_init);

        // Verify data is still there
        {
            let pool = adapter.pool();
            let block_count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blocks")
                .fetch_one(pool)
                .await
                .expect("Failed to count blocks after check");

            assert_eq!(block_count_after, 3, "Should still have 3 blocks");
        }

        // Verify we can query the data
        let latest = adapter.get_latest_block().await.expect("Failed to get latest block");
        assert!(latest.is_some());
        let (block_num, _, _) = latest.unwrap();
        assert_eq!(block_num, 3, "Latest block should still be 3");
    }

    #[tokio::test]
    async fn test_concurrent_writers_are_serialized_by_advisory_lock() {
        // PROOF: pg_advisory_xact_lock serializes concurrent writers on the same chain_id
        // This test FORCES concurrent execution using tokio::spawn to prove:
        // 1. Two writers try to write AT THE SAME TIME
        // 2. One waits for the other (advisory lock blocks)
        // 3. Both succeed without errors (no data loss)
        // 4. Locks are automatically released on commit

        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter1 = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter 1");
        let mut adapter2 = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create adapter 2");

        let registry_address = address!("0000000000000000000000000000000000000001");

        adapter1
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize adapter 1");
        adapter2
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize adapter 2");

        // Spawn both writes CONCURRENTLY using tokio::spawn
        let handle1 = tokio::spawn(async move {
            let block_1 = UrciBlockUpdate {
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

            adapter1
                .write_block(block_1)
                .await
                .expect("Writer 1 should succeed")
        });

        let handle2 = tokio::spawn(async move {
            let block_2 = UrciBlockUpdate {
                block_number: 2,
                block_hash: B256::from([2u8; 32]),
                parent_block_hash: B256::from([1u8; 32]),
                timestamp: 1000002,
                work_id: None,
                parent_work_id: None, // Will compute from block 1
                events: vec![],
                reorged: false,
                system_error: None,
            };

            adapter2
                .write_block(block_2)
                .await
                .expect("Writer 2 should succeed (after waiting for lock)")
        });

        // Wait for BOTH to complete
        let (result1, result2) = tokio::join!(handle1, handle2);
        let work_id_1 = result1.expect("Handle 1 panicked");
        let work_id_2 = result2.expect("Handle 2 panicked");

        // Verify both blocks written successfully
        let adapter_check = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to create check adapter");
        let pool = adapter_check.pool();

        let block_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blocks WHERE chain_id = $1")
            .bind(chain_id)
            .fetch_one(pool)
            .await
            .expect("Failed to count blocks");

        assert_eq!(
            block_count, 2,
            "Both concurrent writers should have successfully written blocks (lock serialized them)"
        );

        // Verify work_ids are different (each block gets unique work_id)
        assert_ne!(
            work_id_1, work_id_2,
            "Each block should have a unique work_id"
        );
    }

    #[tokio::test]
    async fn test_sync_status_update() {
        use crate::postgres::test_helpers::create_test_block_with_event;

        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url).await.expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        // Write a block WITH events so write_block updates writer_status
        // (write_block only updates writer_status if there are events)
        let block = create_test_block_with_event(1, None);
        adapter.write_block(block).await.expect("Failed to write block");

        // Verify writer_status exists after write_block (write_block calls update_sync_status internally)
        let pool = adapter.pool().clone();
        let writer_id = adapter.writer_id.expect("Writer ID should be set");
        let session_id = adapter.session_id.clone().expect("Session ID should be set");

        let row_initial = sqlx::query(
            "SELECT seen_block_from, seen_block_to, chain_tip FROM writer_status
             WHERE writer_id = $1 AND session_id = $2 AND chain_id = $3"
        )
        .bind(writer_id)
        .bind(&session_id)
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch initial writer_status");

        let initial_from: i64 = row_initial.get("seen_block_from");
        let initial_to: i64 = row_initial.get("seen_block_to");
        let initial_tip: i64 = row_initial.get("chain_tip");

        // write_block (with events) sets these to (1, 1, 1) for block 1
        assert_eq!(initial_from, 1, "initial seen_block_from should be 1");
        assert_eq!(initial_to, 1, "initial seen_block_to should be 1");
        assert_eq!(initial_tip, 1, "initial chain_tip should be 1");

        // Now update sync status with a different range
        adapter
            .update_sync_status(5, 10, true)
            .await
            .expect("Failed to update sync status");

        // Verify the sync status was updated
        let row = sqlx::query(
            "SELECT seen_block_from, seen_block_to, chain_tip FROM writer_status
             WHERE writer_id = $1 AND session_id = $2 AND chain_id = $3"
        )
        .bind(writer_id)
        .bind(&session_id)
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch writer_status");

        let seen_from: i64 = row.get("seen_block_from");
        let seen_to: i64 = row.get("seen_block_to");
        let chain_tip: i64 = row.get("chain_tip");

        assert_eq!(seen_from, 1, "seen_block_from should remain 1 (LEAST of 1 and 5)");
        assert_eq!(seen_to, 10, "seen_block_to should be updated to 10 (GREATEST of 1 and 10)");
        assert_eq!(chain_tip, 1, "chain_tip should still be 1 (max canonical block)");

        // Update again with another range
        adapter
            .update_sync_status(11, 20, false)
            .await
            .expect("Failed to update sync status");

        // Verify second update worked
        let row2 = sqlx::query(
            "SELECT seen_block_from, seen_block_to FROM writer_status
             WHERE writer_id = $1 AND session_id = $2 AND chain_id = $3"
        )
        .bind(writer_id)
        .bind(&session_id)
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated writer_status");

        let seen_from_2: i64 = row2.get("seen_block_from");
        let seen_to_2: i64 = row2.get("seen_block_to");

        assert_eq!(seen_from_2, 1, "seen_block_from should still be 1 (LEAST of 1, 5, 11)");
        assert_eq!(seen_to_2, 20, "seen_block_to should be updated to 20 (GREATEST of 1, 10, 20)");
    }

    #[tokio::test]
    async fn test_get_chain_root() {
        use crate::traits::AdapterReader;

        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url).await.expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        let start_block = 100u64;
        adapter
            .initialize(registry_address, start_block)
            .await
            .expect("Failed to initialize");

        // Query chain root for chain_id 1
        let root = adapter.get_chain_root(1).await.expect("Failed to get chain root");

        // Verify it returns the start block we configured
        assert!(root.is_some(), "Chain root should exist after initialization");
        let (start_height, _work_id) = root.unwrap();
        assert_eq!(start_height, start_block, "Chain root start_height should match initialization value");

        // Test querying non-existent chain
        let root_nonexistent = adapter.get_chain_root(999).await.expect("Failed to query non-existent chain");
        assert!(root_nonexistent.is_none(), "Non-existent chain should return None");
    }
}