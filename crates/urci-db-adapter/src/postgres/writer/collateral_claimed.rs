use eyre::Result;
use sqlx::Transaction;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    pub(super) async fn write_collateral_claimed_event_internal(
        &mut self,
        event: urci_common::CollateralClaimed,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        // Insert collateral change record (negative delta for claim)
        let amount_negative = format!("-{}", event.collateralWei);

        sqlx::query(sql_constants::INSERT_OPERATOR_COLLATERAL_DELTA)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(amount_negative)
            .bind(event_id)
            .bind(self.writer_id)
            .execute(tx.as_mut())
            .await?;

        // Insert operator_data record for collateral claimed event
        sqlx::query(sql_constants::INSERT_OPERATOR_DATA_COLLATERAL_CLAIMED)
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
    async fn test_write_collateral_claimed_event() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration, collateral added, then collateral claimed
        let fixture = TestFixture::new()
            .with_registration()
            .with_collateral()
            .with_collateral_claimed();

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Count expected collateral records from fixture data
        // Registration events create 1 collateral record (if collateralWei > 0)
        // CollateralAdded events create 1 collateral record each
        // CollateralClaimed events create 1 collateral record each
        let expected_count = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx| &tx.urc_events)
            .filter(|e| match &e.event {
                urci_common::UrciEventKind::Registration(_, reg_event, _) if reg_event.collateralWei > alloy_primitives::U256::ZERO => true,
                urci_common::UrciEventKind::CollateralAdded(_) => true,
                urci_common::UrciEventKind::CollateralClamed(_) => true,
                _ => false,
            })
            .count();

        // Verify collateral events were recorded
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM operator_collateral WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count as usize, expected_count, "Should have {} collateral records (registration + added + claimed)", expected_count);

        // Verify we have a negative delta for the claimed event
        let has_negative: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM operator_collateral WHERE chain_id = $1 AND collateral_wei_delta < 0)"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(has_negative, "Should have at least one collateral record with negative delta");
    }
}
