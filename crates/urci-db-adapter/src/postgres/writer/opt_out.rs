use eyre::Result;
use sqlx::Transaction;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    pub(super) async fn write_opt_out_event_internal(
        &mut self,
        event: urci_common::OperatorOptedOut,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        // Insert new record with opted_out_at for history
        sqlx::query(sql_constants::INSERT_OPERATOR_SLASHER_COMMITMENT_OPTOUT_SELECT)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(self.writer_id)
            .bind(event.slasher.as_slice())
            .execute(tx.as_mut())
            .await?;

        // Insert operator_data record for audit
        sqlx::query(sql_constants::INSERT_OPERATOR_DATA_OPTED_OUT)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(serde_json::to_value(&event)?)
            .bind(event_id)
            .bind(self.writer_id)
            .execute(tx.as_mut())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::postgres::adapter::PostgresAdapter;
    use crate::postgres::test_helpers::{setup_test_db, TestFixture};
    use crate::traits::{AdapterWriter, Db};
    use alloy_primitives::address;

    #[tokio::test]
    async fn test_write_opt_out_event() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration, opt-in, then opt-out
        let fixture = TestFixture::new()
            .with_registration()
            .with_commitment_opt_in()
            .with_commitment_opt_out();

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Count expected commitment records from fixture data
        // OptIn events create 1 commitment record each
        // OptOut events create 1 NEW commitment record each (with opted_out_at set)
        let expected_count = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .filter(|e| matches!(e.event, urci_common::UrciEventKind::OptIn(_) | urci_common::UrciEventKind::OptOut(_)))
            .count();

        // Verify opt-out was recorded - should have commitment records for both opt-in and opt-out
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM operator_slasher_commitment WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count as usize, expected_count, "Should have {} commitment records (opt-in + opt-out)", expected_count);

        // Verify the latest one has opted_out_at set
        let opted_out_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
            "SELECT opted_out_at FROM operator_slasher_commitment WHERE chain_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(opted_out_at.is_some(), "Latest commitment should have opted_out_at set");
    }
}
