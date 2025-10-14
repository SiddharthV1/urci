use eyre::Result;
use sqlx::Transaction;
use uuid::Uuid;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    pub(super) async fn write_slashing_event_internal(
        &mut self,
        event: urci_common::OperatorSlashed,
        amount: urci_common::SlashAmountWei,
        call: urci_common::SlashingCall,
        event_id: i64,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        // Calculate reward and burn amounts based on slashing type:
        // - Fraud/Equivocation: event.slashAmountWei is the REWARD amount
        //   Contract calls _rewardAndBurn(minCollateralWei / 2, challenger)
        //   Reward = event.slashAmountWei, Burned = event.slashAmountWei
        // - Commitment (both types): event.slashAmountWei is the FULL amount, ALL burned
        //   Contract calls _burnETH(slashAmountWei)
        //   Reward = 0, Burned = event.slashAmountWei
        let (reward_wei, burned_wei) = match &call {
            urci_common::SlashingCall::Fraud { .. }
            | urci_common::SlashingCall::Equivocation { .. } => {
                // Reward and burn are equal (event value is the reward portion)
                (amount, amount)
            },
            urci_common::SlashingCall::Commitment { .. }
            | urci_common::SlashingCall::SlasherCommitment { .. } => {
                // No reward, full amount is burned
                (urci_common::SlashAmountWei::ZERO, amount)
            },
        };

        // Ensure addresses exist
        for (addr, is_contract) in [
            (event.owner.as_slice(), None),         // Owner - unknown
            (event.challenger.as_slice(), None),    // Challenger - unknown
            (event.slasher.as_slice(), Some(true)), // Slasher is a contract
        ] {
            sqlx::query(sql_constants::INSERT_ADDRESS)
                .bind(addr)
                .bind(self.chain_id)
                .bind(is_contract)
                .execute(tx.as_mut())
                .await?;
        }

        // Insert operator_data record first to get ID
        let operator_data_id: Uuid =
            sqlx::query_scalar(sql_constants::INSERT_OPERATOR_DATA_SLASHED)
                .bind(event.registrationRoot.as_slice())
                .bind(self.chain_id)
                .bind(serde_json::to_value(&event)?)
                .bind(event_id)
                .bind(matches!(
                    call,
                    urci_common::SlashingCall::Equivocation { .. }
                ))
                .bind(event_id)
                .bind(self.writer_id)
                .fetch_one(tx.as_mut())
                .await?;

        // Handle different slash types
        match &call {
            urci_common::SlashingCall::Fraud {
                registration_proof,
            } => {
                // Insert BLS pubkey
                let _pubkey_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                        .bind(serde_json::to_value(&registration_proof.registration.pubkey)?)
                        .bind(registration_proof.registration.pubkey.x_a.as_slice())
                        .bind(registration_proof.registration.pubkey.x_b.as_slice())
                        .bind(registration_proof.registration.pubkey.y_a.as_slice())
                        .bind(registration_proof.registration.pubkey.y_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert BLS signature
                let _signature_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_SIGNATURE_G2_RETURNING_ID)
                        .bind(serde_json::to_value(&registration_proof.registration.signature)?)
                        .bind(registration_proof.registration.signature.x_c0_a.as_slice())
                        .bind(registration_proof.registration.signature.x_c0_b.as_slice())
                        .bind(registration_proof.registration.signature.x_c1_a.as_slice())
                        .bind(registration_proof.registration.signature.x_c1_b.as_slice())
                        .bind(registration_proof.registration.signature.y_c0_a.as_slice())
                        .bind(registration_proof.registration.signature.y_c0_b.as_slice())
                        .bind(registration_proof.registration.signature.y_c1_a.as_slice())
                        .bind(registration_proof.registration.signature.y_c1_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Store registration as signed_commitment (Fraud proves fraudulent registration)
                let signed_commitment_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_SIGNED_COMMITMENT)
                        .bind(0i32) // commitment_type: 0 for registration fraud
                        .bind(&[] as &[u8]) // payload: empty for fraud
                        .bind(event.slasher.as_slice())
                        .bind(self.chain_id)
                        .bind(serde_json::to_value(&registration_proof.registration)?)
                        .bind(event_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                sqlx::query(sql_constants::INSERT_SLASH_REGISTRATION)
                    .bind(event_id)
                    .bind(event.registrationRoot.as_slice())
                    .bind(self.chain_id)
                    .bind(operator_data_id)
                    .bind(signed_commitment_id)
                    .bind(event.challenger.as_slice())
                    .bind(reward_wei.to_string())
                    .bind(burned_wei.to_string())
                    .bind(self.writer_id)
                    .execute(tx.as_mut())
                    .await?;
            }

            urci_common::SlashingCall::Commitment {
                registration_proof: _,
                delegation,
                commitment,
            } => {
                // Insert delegation proposer (G1Point)
                let proposer_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation.delegation.proposer)?)
                        .bind(delegation.delegation.proposer.x_a.as_slice())
                        .bind(delegation.delegation.proposer.x_b.as_slice())
                        .bind(delegation.delegation.proposer.y_a.as_slice())
                        .bind(delegation.delegation.proposer.y_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert delegation delegate (G1Point)
                let delegate_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation.delegation.delegate)?)
                        .bind(delegation.delegation.delegate.x_a.as_slice())
                        .bind(delegation.delegation.delegate.x_b.as_slice())
                        .bind(delegation.delegation.delegate.y_a.as_slice())
                        .bind(delegation.delegation.delegate.y_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert delegation signature (G2Point)
                let delegation_signature_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_SIGNATURE_G2_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation.signature)?)
                        .bind(delegation.signature.x_c0_a.as_slice())
                        .bind(delegation.signature.x_c0_b.as_slice())
                        .bind(delegation.signature.x_c1_a.as_slice())
                        .bind(delegation.signature.x_c1_b.as_slice())
                        .bind(delegation.signature.y_c0_a.as_slice())
                        .bind(delegation.signature.y_c0_b.as_slice())
                        .bind(delegation.signature.y_c1_a.as_slice())
                        .bind(delegation.signature.y_c1_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert signed_delegation
                let signed_delegation_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_SIGNED_DELEGATION)
                        .bind(self.chain_id)
                        .bind(proposer_id)
                        .bind(delegate_id)
                        .bind(delegation.delegation.committer.as_slice())
                        .bind(delegation.delegation.slot as i64)
                        .bind(delegation.delegation.metadata.as_ref())
                        .bind(serde_json::to_value(&delegation.delegation)?)
                        .bind(delegation_signature_id)
                        .bind(event_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert signed_commitment
                let signed_commitment_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_SIGNED_COMMITMENT)
                        .bind(commitment.commitment.commitmentType as i32)
                        .bind(commitment.commitment.payload.as_ref())
                        .bind(commitment.commitment.slasher.as_slice())
                        .bind(self.chain_id)
                        .bind(serde_json::to_value(&commitment.commitment)?)
                        .bind(event_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert committer
                let committer_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_COMMITTER_VARIANT)
                        .bind(commitment.commitment.slasher.as_slice())
                        .bind(self.chain_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                sqlx::query(sql_constants::INSERT_SLASH_OFFCHAIN_DELEGATION_COMMITMENT)
                    .bind(self.chain_id)
                    .bind(operator_data_id)
                    .bind(signed_delegation_id)
                    .bind(signed_commitment_id)
                    .bind(committer_id)
                    .bind(event.challenger.as_slice())
                    .bind(reward_wei.to_string())
                    .bind(burned_wei.to_string())
                    .bind(self.writer_id)
                    .execute(tx.as_mut())
                    .await?;
            }

            urci_common::SlashingCall::SlasherCommitment {
                registration_root: _,
                commitment,
                evidence,
            } => {
                // Insert signed commitment
                let signed_commitment_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_SIGNED_COMMITMENT)
                        .bind(commitment.commitment.commitmentType as i32)
                        .bind(commitment.commitment.payload.as_ref())
                        .bind(commitment.commitment.slasher.as_slice())
                        .bind(self.chain_id)
                        .bind(serde_json::to_value(&commitment.commitment)?)
                        .bind(event_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert committer
                let committer_id: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_COMMITTER_VARIANT)
                        .bind(commitment.commitment.slasher.as_slice())
                        .bind(self.chain_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                sqlx::query(sql_constants::INSERT_SLASH_COMMITMENT_VARIANT)
                    .bind(self.chain_id)
                    .bind(operator_data_id)
                    .bind(Some(signed_commitment_id))
                    .bind(committer_id)
                    .bind(event.challenger.as_slice())
                    .bind(evidence.as_slice())
                    .bind(reward_wei.to_string())
                    .bind(burned_wei.to_string())
                    .bind(self.writer_id)
                    .execute(tx.as_mut())
                    .await?;
            }

            urci_common::SlashingCall::Equivocation {
                registration_proof: _,
                delegation_one,
                delegation_two,
            } => {
                // Insert first delegation's proposer (G1Point)
                let proposer_id_1: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation_one.delegation.proposer)?)
                        .bind(delegation_one.delegation.proposer.x_a.as_slice())
                        .bind(delegation_one.delegation.proposer.x_b.as_slice())
                        .bind(delegation_one.delegation.proposer.y_a.as_slice())
                        .bind(delegation_one.delegation.proposer.y_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert first delegation's delegate (G1Point)
                let delegate_id_1: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation_one.delegation.delegate)?)
                        .bind(delegation_one.delegation.delegate.x_a.as_slice())
                        .bind(delegation_one.delegation.delegate.x_b.as_slice())
                        .bind(delegation_one.delegation.delegate.y_a.as_slice())
                        .bind(delegation_one.delegation.delegate.y_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert first delegation's signature (G2Point)
                let signature_id_1: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_SIGNATURE_G2_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation_one.signature)?)
                        .bind(delegation_one.signature.x_c0_a.as_slice())
                        .bind(delegation_one.signature.x_c0_b.as_slice())
                        .bind(delegation_one.signature.x_c1_a.as_slice())
                        .bind(delegation_one.signature.x_c1_b.as_slice())
                        .bind(delegation_one.signature.y_c0_a.as_slice())
                        .bind(delegation_one.signature.y_c0_b.as_slice())
                        .bind(delegation_one.signature.y_c1_a.as_slice())
                        .bind(delegation_one.signature.y_c1_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert first signed_delegation
                let delegation_id_1: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_SIGNED_DELEGATION)
                        .bind(self.chain_id)
                        .bind(proposer_id_1)
                        .bind(delegate_id_1)
                        .bind(delegation_one.delegation.committer.as_slice())
                        .bind(delegation_one.delegation.slot as i64)
                        .bind(delegation_one.delegation.metadata.as_ref())
                        .bind(serde_json::to_value(&delegation_one.delegation)?)
                        .bind(signature_id_1)
                        .bind(event_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert second delegation's proposer (G1Point)
                let proposer_id_2: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation_two.delegation.proposer)?)
                        .bind(delegation_two.delegation.proposer.x_a.as_slice())
                        .bind(delegation_two.delegation.proposer.x_b.as_slice())
                        .bind(delegation_two.delegation.proposer.y_a.as_slice())
                        .bind(delegation_two.delegation.proposer.y_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert second delegation's delegate (G1Point)
                let delegate_id_2: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_PUBKEY_G1_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation_two.delegation.delegate)?)
                        .bind(delegation_two.delegation.delegate.x_a.as_slice())
                        .bind(delegation_two.delegation.delegate.x_b.as_slice())
                        .bind(delegation_two.delegation.delegate.y_a.as_slice())
                        .bind(delegation_two.delegation.delegate.y_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert second delegation's signature (G2Point)
                let signature_id_2: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_BLS_SIGNATURE_G2_RETURNING_ID)
                        .bind(serde_json::to_value(&delegation_two.signature)?)
                        .bind(delegation_two.signature.x_c0_a.as_slice())
                        .bind(delegation_two.signature.x_c0_b.as_slice())
                        .bind(delegation_two.signature.x_c1_a.as_slice())
                        .bind(delegation_two.signature.x_c1_b.as_slice())
                        .bind(delegation_two.signature.y_c0_a.as_slice())
                        .bind(delegation_two.signature.y_c0_b.as_slice())
                        .bind(delegation_two.signature.y_c1_a.as_slice())
                        .bind(delegation_two.signature.y_c1_b.as_slice())
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                // Insert second signed_delegation
                let delegation_id_2: Uuid =
                    sqlx::query_scalar(sql_constants::INSERT_SIGNED_DELEGATION)
                        .bind(self.chain_id)
                        .bind(proposer_id_2)
                        .bind(delegate_id_2)
                        .bind(delegation_two.delegation.committer.as_slice())
                        .bind(delegation_two.delegation.slot as i64)
                        .bind(delegation_two.delegation.metadata.as_ref())
                        .bind(serde_json::to_value(&delegation_two.delegation)?)
                        .bind(signature_id_2)
                        .bind(event_id)
                        .bind(self.writer_id)
                        .fetch_one(tx.as_mut())
                        .await?;

                sqlx::query(sql_constants::INSERT_SLASH_EQUIVOCATION_VARIANT)
                    .bind(self.chain_id)
                    .bind(operator_data_id)
                    .bind(delegation_id_1)
                    .bind(delegation_id_2)
                    .bind(event.challenger.as_slice())
                    .bind(reward_wei.to_string())
                    .bind(burned_wei.to_string())
                    .bind(self.writer_id)
                    .execute(tx.as_mut())
                    .await?;
            }
        }

        // Update operator collateral
        let amount_negative = format!("-{}", amount);
        sqlx::query(sql_constants::INSERT_OPERATOR_COLLATERAL_DELTA)
            .bind(event.registrationRoot.as_slice())
            .bind(self.chain_id)
            .bind(amount_negative)
            .bind(event_id)
            .bind(self.writer_id)
            .execute(tx.as_mut())
            .await?;

        // Mark the operator's commitment as slashed (ONLY for SlasherCommitment type)
        // This matches Registry.sol line 354 which only sets slasherCommitment.slashed = true
        // for the slashCommitment(bytes32, SignedCommitment, bytes) variant.
        // Other slash types (Fraud, Commitment with delegation, Equivocation) do NOT set this flag.
        if let urci_common::SlashingCall::SlasherCommitment { commitment, .. } = &call {
            sqlx::query(sql_constants::UPDATE_OPERATOR_SLASHER_COMMITMENT_SLASHED)
                .bind(event.registrationRoot.as_slice())
                .bind(self.chain_id)
                .bind(commitment.commitment.slasher.as_slice())
                .execute(tx.as_mut())
                .await?;
        }

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
    async fn test_write_slashing_event() {
        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration, opt-in, then slashing
        let fixture = TestFixture::new()
            .with_registration()
            .with_commitment_opt_in()
            .with_slashing();

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Count expected slashing events from fixture data
        let expected_count = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .filter(|e| matches!(e.event, urci_common::UrciEventKind::Slashing(..)))
            .count();

        // Verify slashing was recorded (SlasherCommitment type writes to slash_commitment table)
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM slash_commitment WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count as usize, expected_count, "Should have {} slash_commitment record(s)", expected_count);

        // Verify the commitment was marked as slashed
        let slashed: Option<bool> = sqlx::query_scalar(
            "SELECT slashed FROM operator_slasher_commitment WHERE chain_id = $1 LIMIT 1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(slashed, Some(true), "Commitment should be marked as slashed");
    }

    #[tokio::test]
    async fn test_slashing_reward_and_burn_amounts_slasher_commitment() {
        
        use sqlx::Row;

        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration, opt-in, then slashing
        let fixture = TestFixture::new()
            .with_registration()
            .with_commitment_opt_in()
            .with_slashing();

        // Extract expected values from fixture
        let expected_slash_amount = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .find_map(|e| match &e.event {
                urci_common::UrciEventKind::Slashing(_event, amount, _call) => Some(*amount),
                _ => None,
            })
            .expect("Should have slashing event");

        let initial_collateral = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .find_map(|e| match &e.event {
                urci_common::UrciEventKind::Registration(_addr, event, _result) => Some(event.collateralWei),
                _ => None,
            })
            .expect("Should have registration event");

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Verify reward and burn amounts for SlasherCommitment type
        // SlasherCommitment should have: reward=0, burned=full_amount
        let pool = adapter.pool.clone();
        let row = sqlx::query(
            "SELECT sender_reward_wei::TEXT as reward, burned_wei::TEXT as burned
             FROM slash_commitment WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch slash_commitment");

        let reward_wei: String = row.get("reward");
        let burned_wei: String = row.get("burned");

        // For SlasherCommitment: reward = 0, burned = full amount
        assert_eq!(reward_wei, "0", "SlasherCommitment should have zero reward");
        assert_eq!(
            burned_wei,
            expected_slash_amount.to_string(),
            "SlasherCommitment should burn full amount"
        );

        // Verify that collateral delta entry for slashing exists
        // Database stores deltas as signed NUMERIC (can be negative)
        let slash_delta: String = sqlx::query_scalar(
            "SELECT collateral_wei_delta::TEXT FROM operator_collateral
             WHERE registration_root = $1 AND chain_id = $2
             AND collateral_wei_delta < 0
             ORDER BY id DESC
             LIMIT 1"
        )
        .bind(fixture.registration_root.as_slice())
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch slashing collateral delta");

        // The delta should be negative slash amount
        let expected_delta = format!("-{}", expected_slash_amount);
        assert_eq!(
            slash_delta,
            expected_delta,
            "Collateral delta should be negative slash amount"
        );

        // Verify net collateral: initial - slashed
        let net_collateral: String = sqlx::query_scalar(
            "SELECT SUM(collateral_wei_delta)::TEXT FROM operator_collateral
             WHERE registration_root = $1 AND chain_id = $2"
        )
        .bind(fixture.registration_root.as_slice())
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch net collateral");

        let expected_net = initial_collateral - expected_slash_amount;
        assert_eq!(
            net_collateral,
            expected_net.to_string(),
            "Net collateral should be initial ({}) minus slashed amount ({})",
            initial_collateral,
            expected_slash_amount
        );
    }

    #[tokio::test]
    async fn test_fraud_slashing_with_bls_proof() {
        use sqlx::Row;

        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration + fraud slashing
        let fixture = TestFixture::new()
            .with_registration()
            .with_fraud_slashing();

        // Extract expected slash amount
        let expected_slash_amount = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .find_map(|e| match &e.event {
                urci_common::UrciEventKind::Slashing(_event, amount, _call) => Some(*amount),
                _ => None,
            })
            .expect("Should have slashing event");

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Verify slash_registration was recorded
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM slash_registration WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count slash_registration");

        assert_eq!(count, 1, "Should have 1 slash_registration record");

        // Verify reward and burn amounts for Fraud type
        // Fraud should have: reward=amount, burned=amount (split 50/50)
        let row = sqlx::query(
            "SELECT sender_reward_wei::TEXT as reward, burned_wei::TEXT as burned,
                    signed_commitment_id
             FROM slash_registration WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch slash_registration");

        let reward_wei: String = row.get("reward");
        let burned_wei: String = row.get("burned");
        let signed_commitment_id: Option<uuid::Uuid> = row.get("signed_commitment_id");

        // For Fraud: reward = burned = slash amount
        assert_eq!(reward_wei, expected_slash_amount.to_string(), "Fraud should have reward = slash amount");
        assert_eq!(burned_wei, expected_slash_amount.to_string(), "Fraud should have burned = slash amount");

        // Verify signed_commitment was created
        assert!(signed_commitment_id.is_some(), "Fraud should have signed_commitment_id");

        // Verify the signed_commitment record exists with BLS data
        let commitment_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM signed_commitment WHERE id = $1"
        )
        .bind(signed_commitment_id.unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to count signed_commitment");

        assert_eq!(commitment_count, 1, "signed_commitment record should exist");
    }

    #[tokio::test]
    async fn test_commitment_slashing_with_signed_delegation() {
        use sqlx::Row;

        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration + Commitment slashing
        let fixture = TestFixture::new()
            .with_registration()
            .with_commitment_opt_in()
            .with_commitment_slashing();

        // Extract expected slash amount
        let expected_slash_amount = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .find_map(|e| match &e.event {
                urci_common::UrciEventKind::Slashing(_event, amount, _call) => Some(*amount),
                _ => None,
            })
            .expect("Should have slashing event");

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Verify slash_offchain_delegation_commitment was recorded
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM slash_offchain_delegation_commitment WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count slash_offchain_delegation_commitment");

        assert_eq!(count, 1, "Should have 1 slash_offchain_delegation_commitment record");

        // Verify reward and burn amounts for Commitment type
        // Commitment should have: reward=0, burned=full_amount (NO reward, all burned)
        let row = sqlx::query(
            "SELECT sender_reward_wei::TEXT as reward, burned_wei::TEXT as burned,
                    signed_delegation_id, signed_commitment_id
             FROM slash_offchain_delegation_commitment WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch slash_offchain_delegation_commitment");

        let reward_wei: String = row.get("reward");
        let burned_wei: String = row.get("burned");
        let signed_delegation_id: Option<uuid::Uuid> = row.get("signed_delegation_id");
        let signed_commitment_id: Option<uuid::Uuid> = row.get("signed_commitment_id");

        // For Commitment: reward = 0, burned = full amount
        assert_eq!(reward_wei, "0", "Commitment should have zero reward");
        assert_eq!(burned_wei, expected_slash_amount.to_string(), "Commitment should burn full amount");

        // Verify signed_delegation and signed_commitment were created
        assert!(signed_delegation_id.is_some(), "Commitment should have signed_delegation_id");
        assert!(signed_commitment_id.is_some(), "Commitment should have signed_commitment_id");

        // Verify the signed_delegation record exists with BLS data
        let delegation_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM signed_delegation WHERE id = $1"
        )
        .bind(signed_delegation_id.unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to count signed_delegation");

        assert_eq!(delegation_count, 1, "signed_delegation record should exist");

        // Verify the signed_commitment record exists
        let commitment_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM signed_commitment WHERE id = $1"
        )
        .bind(signed_commitment_id.unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to count signed_commitment");

        assert_eq!(commitment_count, 1, "signed_commitment record should exist");
    }

    #[tokio::test]
    async fn test_equivocation_slashing_with_two_delegations() {
        use sqlx::Row;

        let (_container, db_url) = setup_test_db().await;
        let chain_id = 1i64;

        let mut adapter = PostgresAdapter::connect_new(chain_id, &db_url)
            .await
            .expect("Failed to connect");

        let registry_address = address!("0000000000000000000000000000000000000001");
        adapter.initialize(registry_address, 1).await.expect("Failed to initialize");

        // Create fixture with registration + Equivocation slashing
        let fixture = TestFixture::new()
            .with_registration()
            .with_commitment_opt_in()
            .with_equivocation_slashing();

        // Extract expected slash amount
        let expected_slash_amount = fixture.blocks.iter()
            .flat_map(|b| &b.events)
            .flat_map(|tx_kind| &tx_kind.as_tx_event().urc_events)
            .find_map(|e| match &e.event {
                urci_common::UrciEventKind::Slashing(_event, amount, _call) => Some(*amount),
                _ => None,
            })
            .expect("Should have slashing event");

        // Write all blocks
        for block in &fixture.blocks {
            adapter.write_block(block.clone()).await.expect("Failed to write block");
        }

        // Verify slash_equivocation was recorded
        let pool = adapter.pool.clone();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM slash_equivocation WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count slash_equivocation");

        assert_eq!(count, 1, "Should have 1 slash_equivocation record");

        // Verify reward and burn amounts for Equivocation type
        // Equivocation should have: reward=amount, burned=amount (split 50/50 like Fraud)
        let row = sqlx::query(
            "SELECT sender_reward_wei::TEXT as reward, burned_wei::TEXT as burned,
                    signed_delegation_id_1, signed_delegation_id_2
             FROM slash_equivocation WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch slash_equivocation");

        let reward_wei: String = row.get("reward");
        let burned_wei: String = row.get("burned");
        let signed_delegation_id_1: Option<uuid::Uuid> = row.get("signed_delegation_id_1");
        let signed_delegation_id_2: Option<uuid::Uuid> = row.get("signed_delegation_id_2");

        // For Equivocation: reward = burned = slash amount (50/50 split)
        assert_eq!(reward_wei, expected_slash_amount.to_string(), "Equivocation should have reward = slash amount");
        assert_eq!(burned_wei, expected_slash_amount.to_string(), "Equivocation should have burned = slash amount");

        // Verify both signed_delegations were created
        assert!(signed_delegation_id_1.is_some(), "Equivocation should have signed_delegation_id_1");
        assert!(signed_delegation_id_2.is_some(), "Equivocation should have signed_delegation_id_2");

        // Verify the first signed_delegation record exists
        let delegation_count_1: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM signed_delegation WHERE id = $1"
        )
        .bind(signed_delegation_id_1.unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to count signed_delegation_1");

        assert_eq!(delegation_count_1, 1, "signed_delegation_1 record should exist");

        // Verify the second signed_delegation record exists
        let delegation_count_2: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM signed_delegation WHERE id = $1"
        )
        .bind(signed_delegation_id_2.unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to count signed_delegation_2");

        assert_eq!(delegation_count_2, 1, "signed_delegation_2 record should exist");
    }
}
