use alloy_primitives::B256;
use eyre::Result;
use sqlx::Row;

use super::super::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    /// Helper method to fetch block with events
    pub(super) async fn fetch_block_with_events(
        &self,
        _query_name: &str,
        _chain_id: i64,
        query: sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        use urci_common::api::BlockResponse;

        // Fetch block metadata
        let row = query.fetch_optional(&self.pool).await?;

        let Some(row) = row else {
            return Ok(None);
        };

        // Extract block fields
        let chain_id: i64 = row.get("chain_id");
        let number: i64 = row.get("number");
        let hash: Vec<u8> = row.get("hash");
        let parent_hash: Vec<u8> = row.get("parent_hash");
        let work_id: Vec<u8> = row.get("work_id");
        let parent_work_id: Option<Vec<u8>> = row.get("parent_work_id");
        let timestamp: i64 = row.get("timestamp");
        let canonical: bool = row.get("canonical");
        let finalized: bool = row.get("finalized");
        let head_event_id: Option<i64> = row.get("active_event_id");
        let event_count: i64 = row.get("event_count");

        // Fetch events for this block
        let events = self.fetch_events_for_block(chain_id, &hash).await?;

        Ok(Some(BlockResponse {
            chain_id,
            number,
            hash: B256::from_slice(&hash).to_string(),
            parent_hash: B256::from_slice(&parent_hash).to_string(),
            work_id: B256::from_slice(&work_id).to_string(),
            parent_work_id: parent_work_id.map(|w| B256::from_slice(&w).to_string()),
            timestamp,
            canonical,
            finalized,
            head_event_id,
            event_count,
            events,
            writer_id: None,
        }))
    }

    /// Helper to fetch events for a block
    pub(super) async fn fetch_events_for_block(
        &self,
        chain_id: i64,
        block_hash: &[u8],
    ) -> Result<Vec<urci_common::api::EventResponse>> {
        use urci_common::api::EventResponse;

        let rows = sqlx::query(sql_constants::SELECT_EVENTS_BY_BLOCK)
            .bind(chain_id)
            .bind(block_hash)
            .fetch_all(&self.pool)
            .await?;

        let events = rows
            .into_iter()
            .map(|row| {
                let id: i64 = row.get("id");
                let chain_id: i64 = row.get("chain_id");
                let block_number: i64 = row.get("block_number");
                let block_hash: Vec<u8> = row.get("block_hash");
                let tx_hash: Vec<u8> = row.get("tx_hash");
                let tx_index: i32 = row.get("tx_index");
                let log_index: i32 = row.get("log_index");
                let event_type: String = row.get("event_type");
                let decoded_data: serde_json::Value = row.get("decoded_data");
                let writer_id: Option<uuid::Uuid> = row.get("writer_id");

                EventResponse {
                    id,
                    chain_id,
                    block_number,
                    block_hash: B256::from_slice(&block_hash).to_string(),
                    tx_hash: B256::from_slice(&tx_hash).to_string(),
                    tx_index,
                    log_index,
                    event_type,
                    decoded_data,
                    writer_id: writer_id.map(|w| w.to_string()),
                }
            })
            .collect();

        Ok(events)
    }

    pub(super) async fn get_latest_block_impl(
        &self,
    ) -> Result<Option<(u64, B256, urci_common::WorkId)>> {
        let result =
            sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>)>(sql_constants::SELECT_CANONICAL_BLOCK)
                .fetch_optional(&self.pool)
                .await?;

        Ok(
            result
                .map(|(h, hash, wid)| (h as u64, B256::from_slice(&hash), B256::from_slice(&wid))),
        )
    }

    pub(super) async fn get_latest_finalized_block_impl(
        &self,
    ) -> Result<Option<(u64, B256, urci_common::WorkId)>> {
        let result =
            sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>)>(sql_constants::SELECT_FINALIZED_BLOCK)
                .bind(self.chain_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(
            result
                .map(|(h, hash, wid)| (h as u64, B256::from_slice(&hash), B256::from_slice(&wid))),
        )
    }

    pub(super) async fn get_block_by_number_impl(
        &self,
        block_number: u64,
    ) -> Result<Option<(u64, B256, urci_common::WorkId)>> {
        let result = sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>)>(
            sql_constants::SELECT_BLOCK_BY_NUMBER_CANONICAL,
        )
        .bind(block_number as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            result
                .map(|(h, hash, wid)| (h as u64, B256::from_slice(&hash), B256::from_slice(&wid))),
        )
    }

    pub(super) async fn get_block_by_hash_impl(
        &self,
        block_hash: &B256,
    ) -> Result<Option<(u64, B256, urci_common::WorkId)>> {
        let result = sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>)>(
            sql_constants::SELECT_BLOCK_BY_HASH_CANONICAL,
        )
        .bind(block_hash.as_slice())
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            result
                .map(|(h, hash, wid)| (h as u64, B256::from_slice(&hash), B256::from_slice(&wid))),
        )
    }

    pub(super) async fn get_block_with_events_impl(
        &self,
        chain_id: i64,
        block_number: i64,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.fetch_block_with_events(
            sql_constants::SELECT_BLOCK_WITH_EVENTS_BY_NUMBER,
            chain_id,
            sqlx::query(sql_constants::SELECT_BLOCK_WITH_EVENTS_BY_NUMBER)
                .bind(chain_id)
                .bind(block_number),
        )
        .await
    }

    pub(super) async fn get_block_with_events_by_hash_impl(
        &self,
        chain_id: i64,
        block_hash: &B256,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.fetch_block_with_events(
            sql_constants::SELECT_BLOCK_WITH_EVENTS_BY_HASH,
            chain_id,
            sqlx::query(sql_constants::SELECT_BLOCK_WITH_EVENTS_BY_HASH)
                .bind(chain_id)
                .bind(block_hash.as_slice()),
        )
        .await
    }

    pub(super) async fn get_block_head_impl(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.fetch_block_with_events(
            sql_constants::SELECT_BLOCK_HEAD_WITH_EVENTS,
            chain_id,
            sqlx::query(sql_constants::SELECT_BLOCK_HEAD_WITH_EVENTS).bind(chain_id),
        )
        .await
    }

    pub(super) async fn get_block_finalized_impl(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::BlockResponse>> {
        self.fetch_block_with_events(
            sql_constants::SELECT_BLOCK_FINALIZED_WITH_EVENTS,
            chain_id,
            sqlx::query(sql_constants::SELECT_BLOCK_FINALIZED_WITH_EVENTS).bind(chain_id),
        )
        .await
    }

    pub(super) async fn get_chain_root_impl(
        &self,
        chain_id: u64,
    ) -> Result<Option<(u64, urci_common::WorkId)>> {
        let result =
            sqlx::query_scalar::<_, Option<i64>>(sql_constants::SELECT_CHAIN_ROOT_START_HEIGHT)
                .bind(chain_id as i64)
                .fetch_optional(&self.pool)
                .await?
                .flatten();

        Ok(result.map(|height| (height as u64, B256::default())))
    }

    pub(super) async fn get_fork_status_impl(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::ForkStatusResponse>> {
        use urci_common::api::{ForkHeadResponse, ForkStatusResponse};

        // Get fork status summary
        let status_row = sqlx::query(sql_constants::SELECT_FORK_STATUS)
            .bind(chain_id)
            .fetch_optional(&self.pool)
            .await?;

        let status_row = match status_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let chain_id: i64 = status_row.get("chain_id");
        let highest_common_block: i64 = status_row.get("highest_common_block");
        let forks_detected: bool = status_row.get("forks_detected");
        let total_writers: i32 = status_row.get("total_writers");
        let synced_writers: i32 = status_row.get("synced_writers");

        // If we have blocks, get fork head details
        let fork_heads = if highest_common_block > 0 {
            let fork_head_rows = sqlx::query(sql_constants::SELECT_FORK_HEADS)
                .bind(chain_id)
                .bind(highest_common_block)
                .fetch_all(&self.pool)
                .await?;

            fork_head_rows
                .into_iter()
                .map(|row| {
                    let block_number: i64 = row.get("block_number");
                    let block_hash: Vec<u8> = row.get("block_hash");
                    let head_event_id: Option<i64> = row.get("head_event_id");
                    let canonical: bool = row.get("canonical");
                    let writer_count: i64 = row.get("writer_count");

                    ForkHeadResponse {
                        block_number,
                        block_hash: format!("0x{}", alloy_primitives::hex::encode(block_hash)),
                        head_event_id,
                        canonical,
                        writer_count: writer_count as i32,
                    }
                })
                .collect()
        } else {
            vec![]
        };

        Ok(Some(ForkStatusResponse {
            chain_id,
            highest_common_block,
            forks_detected,
            fork_heads,
            synced_writers,
            total_writers,
        }))
    }

    pub(super) async fn get_contract_config_impl(
        &self,
        chain_id: i64,
    ) -> Result<Option<urci_common::api::ContractConfigResponse>> {
        let row = sqlx::query(sql_constants::SELECT_CONTRACT_CONFIG)
            .bind(chain_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let contract_address: Vec<u8> = row.get("contract_address");
                let registry_address = alloy_primitives::Address::from_slice(&contract_address);

                let min_collateral: String = row.get("min_collateral_wei");
                let fraud_window: i32 = row.get("fraud_proof_window");
                let unreg_delay: i32 = row.get("unregistration_delay");
                let slash_window: i32 = row.get("slash_window");
                let optin_delay: i32 = row.get("opt_in_delay");

                Ok(Some(urci_common::api::ContractConfigResponse {
                    registry_address: format!("{:#x}", registry_address),
                    config: urci_common::api::RegistryConfig {
                        min_collateral_wei: min_collateral,
                        fraud_proof_window_blocks: fraud_window as i64,
                        unregistration_delay_blocks: unreg_delay as i64,
                        slashing_window_blocks: slash_window as i64,
                        opt_in_delay_blocks: optin_delay as i64,
                    },
                }))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::postgres::adapter::PostgresAdapter;
    use crate::postgres::test_helpers::{create_test_block_with_event, setup_test_db};
    use crate::traits::{AdapterWriter, Db};
    use alloy_primitives::address;

    #[tokio::test]
    async fn test_get_block_with_events() {
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

        // Write block with events
        let block_1 = create_test_block_with_event(1, None);
        adapter
            .write_block(block_1.clone())
            .await
            .expect("Failed to write block 1");

        let result = adapter
            .get_block_with_events_impl(chain_id, 1)
            .await
            .expect("Failed to get block");

        assert!(result.is_some());
        let block_response = result.unwrap();

        assert_eq!(block_response.chain_id, chain_id);
        assert_eq!(block_response.number, 1);
        assert!(block_response.canonical);
        assert!(!block_response.finalized);
        assert!(block_response.head_event_id.is_some());
        assert_eq!(block_response.event_count, 1);
        assert_eq!(block_response.events.len(), 1);
        assert_eq!(block_response.events[0].event_type, "OperatorRegistered");
    }

    #[tokio::test]
    async fn test_get_block_with_events_by_hash() {
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

        let block_1 = create_test_block_with_event(1, None);
        let block_hash = block_1.block_hash;
        adapter
            .write_block(block_1)
            .await
            .expect("Failed to write block 1");

        let result = adapter
            .get_block_with_events_by_hash_impl(chain_id, &block_hash)
            .await
            .expect("Failed to get block");

        assert!(result.is_some());
        let block_response = result.unwrap();
        assert_eq!(block_response.number, 1);
        assert_eq!(block_response.hash, block_hash.to_string());
    }

    #[tokio::test]
    async fn test_get_block_head() {
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

        // Write multiple blocks
        let mut last_work_id = None;
        for i in 1..=3 {
            let block = create_test_block_with_event(i, last_work_id);
            adapter
                .write_block(block)
                .await
                .unwrap_or_else(|_| panic!("Failed to write block {}", i));

            let block_info = adapter
                .get_block_by_number_impl(i)
                .await
                .expect("Failed to get block")
                .expect("Block should exist");
            last_work_id = Some(block_info.2);
        }

        let result = adapter
            .get_block_head_impl(chain_id)
            .await
            .expect("Failed to get head");

        assert!(result.is_some());
        let head = result.unwrap();
        assert_eq!(head.number, 3);
        assert!(head.canonical);
    }

    #[tokio::test]
    async fn test_get_block_finalized() {
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

        // Write blocks
        let mut last_work_id = None;
        for i in 1..=3 {
            let block = create_test_block_with_event(i, last_work_id);
            adapter
                .write_block(block)
                .await
                .unwrap_or_else(|_| panic!("Failed to write block {}", i));

            let block_info = adapter
                .get_block_by_number_impl(i)
                .await
                .expect("Failed to get block")
                .expect("Block should exist");
            last_work_id = Some(block_info.2);
        }

        // Finalize blocks 1 and 2
        adapter
            .finalize_blocks(chain_id, 2)
            .await
            .expect("Failed to finalize blocks");

        let result = adapter
            .get_block_finalized_impl(chain_id)
            .await
            .expect("Failed to get finalized");

        assert!(result.is_some());
        let finalized = result.unwrap();
        assert_eq!(finalized.number, 2);
        assert!(finalized.finalized);
    }
}
