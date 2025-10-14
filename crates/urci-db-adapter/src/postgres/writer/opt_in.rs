use eyre::Result;
use sqlx::Transaction;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    #[instrument(skip(self, tx), fields(
        registration_root = %alloy_primitives::hex::encode(event.registrationRoot),
        slasher = %event.slasher,
        committer = %event.committer,
        event_id = event_id
    ))]
    pub(super) async fn write_opt_in_event_internal(
        &mut self,
        event: urci_common::OperatorOptedIn,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        debug!("Writing OptIn event");

        // Ensure slasher and committer addresses exist
        debug!(address = %event.slasher, "Inserting slasher address");
        sqlx::query(sql_constants::INSERT_ADDRESS)
            .bind(event.slasher.as_slice())
            .bind(self.chain_id)
            .bind(Some(true)) // Slasher is a contract
            .execute(tx.as_mut())
            .await
            .map_err(|e| {
                error!(error = %e, address = %event.slasher, "Failed to insert slasher address");
                e
            })?;

        debug!(address = %event.committer, "Inserting committer address");
        sqlx::query(sql_constants::INSERT_ADDRESS)
            .bind(event.committer.as_slice())
            .bind(self.chain_id)
            .bind(Some(true)) // Committer is a contract
            .execute(tx.as_mut())
            .await
            .map_err(|e| {
                error!(error = %e, address = %event.committer, "Failed to insert committer address");
                e
            })?;

        // Insert slasher record
        debug!("Inserting slasher record");
        let slasher_id: Uuid = sqlx::query_scalar(sql_constants::INSERT_SLASHER_RETURNING_ID)
            .bind(self.chain_id)
            .bind(event.slasher.as_slice())
            .bind(event_id)
            .bind(self.writer_id)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to insert slasher record");
                e
            })?;

        debug!(slasher_id = %slasher_id, "Slasher record inserted");

        // Insert committer record
        debug!("Inserting committer record");
        let committer_id: Uuid = sqlx::query_scalar(sql_constants::INSERT_COMMITTER_RETURNING_ID)
            .bind(self.chain_id)
            .bind(event.committer.as_slice())
            .bind(event_id)
            .bind(self.writer_id)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to insert committer record");
                e
            })?;

        debug!(committer_id = %committer_id, "Committer record inserted");

        // Always INSERT for history (not UPSERT)
        debug!(
            slasher_id = %slasher_id,
            committer_id = %committer_id,
            "Inserting operator_slasher_commitment record"
        );
        sqlx::query(sql_constants::INSERT_OPERATOR_SLASHER_COMMITMENT_OPTIN)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(slasher_id)
            .bind(committer_id)
            .bind(self.writer_id)
            .execute(tx.as_mut())
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    slasher_id = %slasher_id,
                    committer_id = %committer_id,
                    "Failed to insert operator_slasher_commitment"
                );
                e
            })?;

        debug!("Operator commitment inserted");

        // Insert operator_data record for audit
        debug!("Inserting operator_data record");
        sqlx::query(sql_constants::INSERT_OPERATOR_DATA_OPTED_IN)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(serde_json::to_value(&event)?)
            .bind(event_id)
            .bind(self.writer_id)
            .execute(tx.as_mut())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to insert operator_data");
                e
            })?;

        info!("OptIn event written successfully");
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
    async fn test_write_opt_in_event() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create comprehensive test fixture with registration + opt-in event
        let fixture = TestFixture::new()
            .with_registration()
            .with_commitment_opt_in();

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Count expected opt-in events from fixture data
        let expected_count = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .filter(|e| matches!(e.event, urci_common::UrciEventKind::OptIn(_)))
            .count();

        // Verify the commitment was recorded
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM operator_slasher_commitment WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count as usize, expected_count, "Should have {} commitment record(s)", expected_count);
    }
}
