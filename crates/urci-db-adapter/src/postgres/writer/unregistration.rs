use eyre::Result;
use sqlx::Transaction;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    pub(super) async fn write_unregistration_event_internal(
        &mut self,
        event: urci_common::OperatorUnregistered,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        // Update operator record to mark as unregistered
        sqlx::query(sql_constants::UPDATE_OPERATOR_UNREGISTER)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .execute(tx.as_mut())
            .await?;

        // Insert operator_data record for unregistration event
        sqlx::query(sql_constants::INSERT_OPERATOR_DATA_UNREGISTERED)
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
    async fn test_write_unregistration_event() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration and unregistration
        let fixture = TestFixture::new()
            .with_registration()
            .with_unregistration();

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Verify unregistration was recorded (registration_processed should be FALSE)
        let pool = adapter.pool.clone();
        let registration_processed: bool = sqlx::query_scalar(
            "SELECT registration_processed FROM operators WHERE registration_root = $1 AND chain_id = $2"
        )
        .bind(fixture.registration_root.as_slice())
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch operator");

        assert!(!registration_processed, "Operator should be marked as unregistered (registration_processed = FALSE)");

        // Verify operator_data record was created for unregistration event
        let event_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE chain_id = $1 AND event_type = 'OperatorUnregistered'"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count unregistration events");

        assert_eq!(event_count, 1, "Should have exactly one OperatorUnregistered event");
    }
}
