use alloy_primitives::{hex, U256};
use eyre::Result;
use sqlx::Transaction;
use uuid::Uuid;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    pub(super) async fn write_registration_event_internal(
        &mut self,
        owner: urci_common::Owner,
        event: urci_common::OperatorRegistered,
        validation: urci_common::RegistrationValidationResult,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        // Ensure owner address exists
        sqlx::query(sql_constants::INSERT_ADDRESS)
            .bind(owner.as_slice())
            .bind(self.chain_id)
            .bind(false) // Default to false, can check bytecode later if needed
            .execute(tx.as_mut())
            .await?;

        // Insert Merkle Tree Record first
        let leaves_json = serde_json::to_value(
            validation
                .tree
                .leaves
                .iter()
                .map(|leaf| format!("0x{}", hex::encode(leaf)))
                .collect::<Vec<_>>(),
        )?;

        sqlx::query(sql_constants::INSERT_MERKLE_TREE)
            .bind(event.registrationRoot.as_slice())
            .bind(leaves_json)
            .bind(validation.tree.leaves.len() as i32)
            .execute(tx.as_mut())
            .await?;

        // Insert one signed_registration per BLS signature validation
        for (idx, sig_validation) in validation.signature.iter().enumerate() {
            // Insert BLS signature G2 point
            let sig_id: Uuid =
                sqlx::query_scalar(sql_constants::INSERT_BLS_SIGNATURE_G2_RETURNING_ID)
                    .bind(serde_json::to_value(&sig_validation.signature_data)?)
                    .bind(sig_validation.signature_data.x_c0_a.as_slice())
                    .bind(sig_validation.signature_data.x_c0_b.as_slice())
                    .bind(sig_validation.signature_data.y_c0_a.as_slice())
                    .bind(sig_validation.signature_data.y_c0_b.as_slice())
                    .bind(sig_validation.signature_data.x_c1_a.as_slice())
                    .bind(sig_validation.signature_data.x_c1_b.as_slice())
                    .bind(sig_validation.signature_data.y_c1_a.as_slice())
                    .bind(sig_validation.signature_data.y_c1_b.as_slice())
                    .bind(self.writer_id)
                    .fetch_one(tx.as_mut())
                    .await?;

            // Insert BLS pubkey G1 point
            let pubkey_id: Uuid =
                sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                    .bind(serde_json::to_value(&sig_validation.pubkey_data)?)
                    .bind(sig_validation.pubkey_data.x_a.as_slice())
                    .bind(sig_validation.pubkey_data.x_b.as_slice())
                    .bind(sig_validation.pubkey_data.y_a.as_slice())
                    .bind(sig_validation.pubkey_data.y_b.as_slice())
                    .bind(self.writer_id)
                    .fetch_one(tx.as_mut())
                    .await?;

            // Insert Merkle Inclusion Proof
            let proof_id: Uuid =
                sqlx::query_scalar(sql_constants::INSERT_MERKLE_PROOF_RETURNING_ID)
                    .bind(idx as i32)
                    .bind(
                        sig_validation
                            .merkle_proof
                            .iter()
                            .map(|h| h.as_slice())
                            .collect::<Vec<_>>(),
                    )
                    .bind(event.registrationRoot.as_slice())
                    .bind(self.writer_id)
                    .fetch_one(tx.as_mut())
                    .await?;

            // Serialize validation_error to JSONB
            let validation_error_json = match &sig_validation.validation_error {
                Some(err) => serde_json::to_value(err)?,
                None => serde_json::Value::Null,
            };

            // Insert owner address into address table (required for foreign key constraint)
            sqlx::query(sql_constants::INSERT_ADDRESS)
                .bind(owner.as_slice())
                .bind(self.chain_id)
                .bind(false) // owner_address is not a contract (it's an EOA)
                .execute(tx.as_mut())
                .await?;

            // Insert Signed Registration Record
            sqlx::query(sql_constants::INSERT_SIGNED_REGISTRATION)
                .bind(owner.as_slice())
                .bind(self.chain_id)
                .bind(sig_id)
                .bind(pubkey_id)
                .bind(event.registrationRoot.as_slice())
                .bind(proof_id)
                .bind(if sig_validation.is_fraudulent {
                    "failed_signature_verification"
                } else {
                    "verified"
                })
                .bind(validation_error_json)
                .bind(self.writer_id)
                .execute(tx.as_mut())
                .await?;
        }

        // Insert Operator Record via root and chain_id
        sqlx::query(sql_constants::INSERT_OPERATOR_IGNORE)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(owner.as_slice())
            .bind(validation.number_of_keys() as i32)
            .bind(!validation.has_fraudulent)
            .bind(event_id)
            .bind(self.writer_id)
            .execute(tx.as_mut())
            .await?;

        // Insert Operator Data Record
        sqlx::query(sql_constants::INSERT_OPERATOR_DATA_REGISTERED)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(serde_json::to_value(&event)?)
            .bind(event_id) // registered_at is event_id reference
            .bind(event_id)
            .bind(self.writer_id)
            .execute(tx.as_mut())
            .await?;

        // Insert Operator Collateral Record if initial collateral provided
        if event.collateralWei > U256::ZERO {
            sqlx::query(sql_constants::INSERT_OPERATOR_COLLATERAL_FULL)
                .bind(event.registrationRoot.as_slice())
                .bind(self.chain_id)
                .bind(event.collateralWei.to_string())
                .bind(event_id)
                .bind(self.writer_id)
                .execute(tx.as_mut())
                .await?;
        }

        // Note: writer_status is updated by write_block() which calls this function
        // No need to update it here as write_block() has the full block context
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::postgres::adapter::PostgresAdapter;
    use crate::postgres::test_helpers::{create_test_block_with_event, setup_test_db};
    use crate::traits::{AdapterWriter, Db};
    use alloy_primitives::address;
    use sqlx::Row;

    #[tokio::test]
    async fn test_write_registration_event() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        // Initialize DB
        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        // Use the writer API to create a block with a registration event
        let block_1 = create_test_block_with_event(1, None);

        // Extract registration_root and expected_leaf_count from the block's event data
        let (registration_root, expected_leaf_count): (alloy_primitives::B256, usize) = if let Some(tx_event_kind) = block_1.events.first() {
            if let Some(urc_event) = tx_event_kind.as_tx_event().urc_events.first() {
                match &urc_event.event {
                    urci_common::UrciEventKind::Registration(_, event, validation_result) => {
                        let leaf_count = validation_result.as_ref()
                            .map(|v| v.tree.leaves.len())
                            .unwrap_or(0);
                        (event.registrationRoot.into(), leaf_count)
                    }
                    _ => panic!("Expected Registration event"),
                }
            } else {
                panic!("Expected at least one URC event");
            }
        } else {
            panic!("Expected at least one transaction event");
        };

        adapter
            .write_block(block_1)
            .await
            .expect("Failed to write block");

        let pool = adapter.pool.clone();

        // Verify the merkle tree was inserted by write_block -> write_registration_event

        let row = sqlx::query(crate::sql_constants::SELECT_MERKLE_TREE_LEAF_COUNT)
            .bind(registration_root.as_slice())
            .fetch_one(&pool)
            .await
            .unwrap();

        let leaf_count: i32 = row.get("leaf_count");
        assert_eq!(
            leaf_count as usize,
            expected_leaf_count,
            "Merkle tree leaf count should match the validation result"
        );
    }

    #[tokio::test]
    async fn test_write_block_with_operator_registered_batch_api() {
        let (_container, db_url) = setup_test_db().await;

        let chain_id = 1i64;
        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url).await.expect("Failed to create adapter");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter
            .initialize(registry_address, 1)
            .await
            .expect("Failed to initialize");

        // Use the existing test helper to create a registration block with validation data
        let block = create_test_block_with_event(1, None);

        // Create batch and write using the batch API (tests write_blocks path)
        let mut batch = urci_common::UrciBatchedBlockRangeUpdate::new();
        batch.add_block(block);

        // Write should now succeed with chain_id properly included
        adapter.write_blocks(batch).await.expect("Write should succeed with chain_id included");

        // Verify registration was written
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM operators WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count operators");

        assert_eq!(count, 1, "Should have one operator registered via batch API");
    }
}
