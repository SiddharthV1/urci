use alloy_primitives::{hex, Address, B256};
use eyre::Result;
use sqlx::Row;

use super::super::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    /// Helper method to map operator row to OperatorResponse
    pub(super) fn map_operator_row(
        &self,
        row: sqlx::postgres::PgRow,
    ) -> urci_common::api::OperatorResponse {
        use sqlx::Row;
        use urci_common::api::OperatorResponse;

        let registration_root: Vec<u8> = row.get("registration_root");
        let chain_id: i64 = row.get("chain_id");
        let owner_address: Vec<u8> = row.get("owner_address");
        let num_keys: Option<i32> = row.get("num_keys");
        let registration_processed: bool = row.get("registration_processed");
        let block_number: Option<i64> = row.get("block_number");
        let collateral_wei_total: Option<sqlx::types::BigDecimal> = row.get("collateral_wei_total");
        let last_event_type: Option<String> = row.get("last_event_type");
        let deleted: Option<bool> = row.get("deleted");
        let equivocated: Option<bool> = row.get("equivocated");
        let canonical: Option<bool> = row.get("canonical");
        let finalized: Option<bool> = row.get("finalized");
        let sync_status: Option<String> = row.get("sync_status");

        OperatorResponse {
            registration_root: format!("0x{}", hex::encode(registration_root)),
            chain_id,
            owner_address: format!("0x{}", hex::encode(owner_address)),
            num_keys,
            registration_processed,
            block_number,
            collateral_wei_total: collateral_wei_total.map(|v| v.to_string()),
            last_event_type,
            deleted,
            equivocated,
            canonical,
            finalized,
            sync_status,
            commitments: None,
            slashing_history: None,
            timeline: None,
        }
    }

    /// Helper method to fetch operator by a specific field (registration_root or owner_address)
    pub(super) async fn fetch_operator_by_field(
        &self,
        field_name: &str,
        chain_id: i64,
        field_value: &[u8],
    ) -> Result<Option<urci_common::api::OperatorResponse>> {
        use urci_common::api::OperatorResponse;

        // Build query with proper JOINs to get all fields from related tables
        let query = format!(
            r#"
            SELECT
                o.registration_root,
                o.chain_id::bigint as chain_id,
                o.owner_address,
                o.num_keys,
                o.registration_processed,
                e.block_number,
                oc.collateral_wei_total,
                od.event_type::text as last_event_type,
                od.deleted,
                od.equivocated,
                b.canonical,
                b.finalized,
                CASE
                    WHEN b.finalized THEN 'finalized'
                    WHEN b.canonical THEN 'canonical'
                    ELSE 'pending'
                END as sync_status
            FROM operators o
            LEFT JOIN events e ON e.id = o.event_id
            LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
            LEFT JOIN LATERAL (
                SELECT collateral_wei_total
                FROM operator_collateral
                WHERE registration_root = o.registration_root
                    AND chain_id = o.chain_id
                ORDER BY created_at DESC
                LIMIT 1
            ) oc ON true
            LEFT JOIN LATERAL (
                SELECT event_type, deleted, equivocated
                FROM operator_data
                WHERE registration_root = o.registration_root
                    AND chain_id = o.chain_id
                ORDER BY created_at DESC
                LIMIT 1
            ) od ON true
            WHERE o.chain_id = $1
                AND o.{} = $2
            LIMIT 1
            "#,
            field_name
        );

        let row = sqlx::query(&query)
            .bind(chain_id)
            .bind(field_value)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let registration_root: Vec<u8> = row.get("registration_root");
        let chain_id: i64 = row.get("chain_id");
        let owner_address: Vec<u8> = row.get("owner_address");
        let num_keys: Option<i32> = row.get("num_keys");
        let registration_processed: bool = row.get("registration_processed");
        let block_number: Option<i64> = row.get("block_number");
        let collateral_wei_total: Option<String> = row.get("collateral_wei_total");
        let last_event_type: Option<String> = row.get("last_event_type");
        let deleted: Option<bool> = row.get("deleted");
        let equivocated: Option<bool> = row.get("equivocated");
        let canonical: Option<bool> = row.get("canonical");
        let finalized: Option<bool> = row.get("finalized");
        let sync_status: Option<String> = row.get("sync_status");

        Ok(Some(OperatorResponse {
            registration_root: B256::from_slice(&registration_root).to_string(),
            chain_id,
            owner_address: Address::from_slice(&owner_address).to_string(),
            num_keys,
            registration_processed,
            block_number,
            collateral_wei_total,
            last_event_type,
            deleted,
            equivocated,
            canonical,
            finalized,
            sync_status,
            commitments: None,
            slashing_history: None,
            timeline: None,
        }))
    }

    pub(super) async fn get_operator_by_registration_root_impl(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorResponse>> {
        self.fetch_operator_by_field("registration_root", chain_id, registration_root.as_slice())
            .await
    }

    pub(super) async fn get_operator_profile_impl(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorResponse>> {
        tracing::debug!(
            chain_id = chain_id,
            registration_root = %hex::encode(registration_root),
            "Getting operator profile"
        );

        // Get basic operator info
        let mut operator = match self
            .get_operator_by_registration_root_impl(chain_id, registration_root)
            .await?
        {
            Some(op) => {
                tracing::debug!("Found operator");
                op
            }
            None => {
                tracing::warn!("Operator not found");
                return Ok(None);
            }
        };

        // Get commitments (optional - don't fail if query fails)
        let commitments = self
            .get_operator_commitments_impl(chain_id, registration_root, 1000, None)
            .await
            .unwrap_or_default();
        operator.commitments = Some(commitments);

        // Get slashing history (optional - don't fail if query fails)
        let slashing = self
            .get_operator_slashing_impl(chain_id, registration_root, 1000, None)
            .await
            .ok()
            .flatten();
        operator.slashing_history = Some(slashing.map(|s| s.slashes).unwrap_or_default());

        // Timeline is optional and not implemented yet
        operator.timeline = Some(vec![]);

        Ok(Some(operator))
    }

    pub(super) async fn get_operator_by_owner_impl(
        &self,
        chain_id: i64,
        owner_address: &Address,
    ) -> Result<Option<urci_common::api::OperatorResponse>> {
        self.fetch_operator_by_field("owner_address", chain_id, owner_address.as_slice())
            .await
    }

    pub(super) async fn list_operators_impl(
        &self,
        chain_id: i64,
        limit: i64,
        _cursor: Option<String>,
    ) -> Result<Vec<urci_common::api::OperatorResponse>> {
        use urci_common::api::OperatorResponse;

        let query = r#"
            SELECT
                o.registration_root,
                o.chain_id::bigint as chain_id,
                o.owner_address,
                o.num_keys,
                o.registration_processed,
                e.block_number,
                oc.collateral_wei_total,
                od.event_type::text as last_event_type,
                od.deleted,
                od.equivocated,
                b.canonical,
                b.finalized,
                CASE
                    WHEN b.finalized THEN 'finalized'
                    WHEN b.canonical THEN 'canonical'
                    ELSE 'pending'
                END as sync_status
            FROM operators o
            LEFT JOIN events e ON e.id = o.event_id
            LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
            LEFT JOIN LATERAL (
                SELECT collateral_wei_total
                FROM operator_collateral
                WHERE registration_root = o.registration_root
                    AND chain_id = o.chain_id
                ORDER BY created_at DESC
                LIMIT 1
            ) oc ON true
            LEFT JOIN LATERAL (
                SELECT event_type, deleted, equivocated
                FROM operator_data
                WHERE registration_root = o.registration_root
                    AND chain_id = o.chain_id
                ORDER BY created_at DESC
                LIMIT 1
            ) od ON true
            WHERE o.chain_id = $1
            ORDER BY e.block_number DESC NULLS LAST, o.registration_root
            LIMIT $2
        "#;

        let rows = sqlx::query(query)
            .bind(chain_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let operators = rows
            .into_iter()
            .map(|row| {
                let registration_root: Vec<u8> = row.get("registration_root");
                let chain_id: i64 = row.get("chain_id");
                let owner_address: Vec<u8> = row.get("owner_address");
                let num_keys: Option<i32> = row.get("num_keys");
                let registration_processed: bool = row.get("registration_processed");
                let block_number: Option<i64> = row.get("block_number");
                let collateral_wei_total: Option<String> = row.get("collateral_wei_total");
                let last_event_type: Option<String> = row.get("last_event_type");
                let deleted: Option<bool> = row.get("deleted");
                let equivocated: Option<bool> = row.get("equivocated");
                let canonical: Option<bool> = row.get("canonical");
                let finalized: Option<bool> = row.get("finalized");
                let sync_status: Option<String> = row.get("sync_status");

                OperatorResponse {
                    registration_root: B256::from_slice(&registration_root).to_string(),
                    chain_id,
                    owner_address: Address::from_slice(&owner_address).to_string(),
                    num_keys,
                    registration_processed,
                    block_number,
                    collateral_wei_total,
                    last_event_type,
                    deleted,
                    equivocated,
                    canonical,
                    finalized,
                    sync_status,
                    commitments: None,
                    slashing_history: None,
                    timeline: None,
                }
            })
            .collect();

        Ok(operators)
    }

    pub(super) async fn search_operator_by_address_impl(
        &self,
        chain_id: i64,
        address: &Address,
    ) -> Result<Option<urci_common::api::OperatorSearchResponse>> {
        use urci_common::api::{BlsKeyResponse, OperatorSearchResponse};

        // First, get the operator by owner address
        let operator = self.get_operator_by_owner_impl(chain_id, address).await?;

        let Some(operator) = operator else {
            return Ok(None);
        };

        // Parse the registration root to query for keys
        let registration_root = operator
            .registration_root
            .strip_prefix("0x")
            .unwrap_or(&operator.registration_root);
        let root_bytes = hex::decode(registration_root)?;
        let root_hash = B256::from_slice(&root_bytes);

        // Get BLS keys for this operator from merkle_inclusion_proof table
        let keys_query = r#"
            SELECT
                mip.merkle_index,
                mip.merkle_inclusion_proof,
                mip.bytes_32
            FROM merkle_inclusion_proof mip
            WHERE mip.registration_root = $1
                AND mip.chain_id = $2
            ORDER BY mip.merkle_index
        "#;

        let key_rows = sqlx::query(keys_query)
            .bind(root_hash.as_slice())
            .bind(chain_id)
            .fetch_all(&self.pool)
            .await?;

        let keys = key_rows
            .into_iter()
            .map(|row| {
                let merkle_index: i32 = row.get("merkle_index");
                let merkle_inclusion_proof: Option<Vec<Vec<u8>>> =
                    row.get("merkle_inclusion_proof");
                let bytes_32: Option<Vec<u8>> = row.get("bytes_32");

                // Convert bytes_32 to hex
                let pubkey_hex = if let Some(bytes) = bytes_32 {
                    format!("0x{}", hex::encode(bytes))
                } else {
                    "0x".to_string()
                };

                // Convert merkle proof bytea array to vec of hex strings
                let proof_array = if let Some(proof_arr) = merkle_inclusion_proof {
                    proof_arr
                        .into_iter()
                        .map(|bytes| format!("0x{}", hex::encode(bytes)))
                        .collect()
                } else {
                    vec![]
                };

                BlsKeyResponse {
                    pubkey_hex,
                    key_index: Some(merkle_index),
                    merkle_proof: Some(proof_array),
                }
            })
            .collect();

        Ok(Some(OperatorSearchResponse { operator, keys }))
    }

    pub(super) async fn find_operators_by_pubkey_impl(
        &self,
        chain_id: i64,
        pubkey_bytes: &[u8],
    ) -> Result<Vec<urci_common::api::OperatorWithPubkeyIndex>> {
        use urci_common::api::OperatorWithPubkeyIndex;

        let rows = sqlx::query(sql_constants::SELECT_OPERATORS_BY_PUBKEY)
            .bind(pubkey_bytes)
            .bind(chain_id)
            .fetch_all(&self.pool)
            .await?;

        let operators = rows
            .into_iter()
            .map(|row| {
                let registration_root: Vec<u8> = row.get("registration_root");
                let chain_id: i64 = row.get("chain_id");
                let merkle_index: i32 = row.get("merkle_index");
                let owner_address: Vec<u8> = row.get("owner_address");

                OperatorWithPubkeyIndex {
                    registration_root: format!("0x{}", hex::encode(registration_root)),
                    chain_id,
                    owner_address: format!("0x{}", hex::encode(owner_address)),
                    key_index: merkle_index,
                }
            })
            .collect();

        Ok(operators)
    }

    pub(super) async fn get_operator_keys_impl(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorKeysResponse>> {
        use urci_common::api::{BlsKeyResponse, OperatorKeysResponse};

        let key_rows = sqlx::query(sql_constants::SELECT_OPERATOR_KEYS)
            .bind(registration_root.as_slice())
            .bind(chain_id)
            .fetch_all(&self.pool)
            .await?;

        // Always return a response (even if empty keys) since the query succeeded
        // Return None only if operator doesn't exist
        let keys: Vec<BlsKeyResponse> = key_rows
            .into_iter()
            .map(|row| {
                let merkle_index: i32 = row.get("merkle_index");
                let merkle_inclusion_proof: Option<Vec<Vec<u8>>> =
                    row.get("merkle_inclusion_proof");
                let bytes_32: Option<Vec<u8>> = row.get("bytes_32");

                let pubkey_hex = if let Some(bytes) = bytes_32 {
                    format!("0x{}", hex::encode(bytes))
                } else {
                    "0x".to_string()
                };

                let proof_array = if let Some(proof_arr) = merkle_inclusion_proof {
                    proof_arr
                        .into_iter()
                        .map(|bytes| format!("0x{}", hex::encode(bytes)))
                        .collect()
                } else {
                    vec![]
                };

                BlsKeyResponse {
                    pubkey_hex,
                    key_index: Some(merkle_index),
                    merkle_proof: Some(proof_array),
                }
            })
            .collect();

        Ok(Some(OperatorKeysResponse {
            registration_root: format!("0x{}", hex::encode(registration_root)),
            chain_id,
            num_keys: keys.len() as i32,
            keys,
        }))
    }

    pub(super) async fn get_operators_at_block_impl(
        &self,
        block_number: u64,
    ) -> Result<Vec<urci_common::OperatorState>> {
        // Get all operators registered at or before this block
        let operator_rows = sqlx::query(
            r#"
            SELECT DISTINCT
                o.registration_root,
                o.owner_address,
                o.num_keys,
                o.registration_processed
            FROM operators o
            JOIN events e ON e.id = o.event_id
            JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
            WHERE o.chain_id = $1
                AND b.number <= $2
                AND b.canonical = true
            "#,
        )
        .bind(self.chain_id)
        .bind(block_number as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut operators = Vec::new();

        for operator_row in operator_rows {
            let registration_root: Vec<u8> = operator_row.get("registration_root");
            let owner_address: Vec<u8> = operator_row.get("owner_address");
            let _num_keys: Option<i32> = operator_row.get("num_keys");
            let _registration_processed: bool = operator_row.get("registration_processed");

            let _registration_root_b256 = B256::from_slice(&registration_root);
            let owner_addr = Address::from_slice(&owner_address);

            // Get operator state at this block
            if let Some(state) = self
                .get_operator_state_impl(&owner_addr, block_number)
                .await?
            {
                operators.push(state);
            }
        }

        Ok(operators)
    }

    pub(super) async fn get_operator_state_impl(
        &self,
        operator: &Address,
        block_number: u64,
    ) -> Result<Option<urci_common::OperatorState>> {
        use alloy_primitives::U256;
        use urci_common::{CommitmentProtocol, OperatorEvent, OperatorEventType, OperatorState};

        // Get operator's registration root
        let operator_row = sqlx::query(
            r#"
            SELECT
                o.registration_root,
                o.owner_address,
                o.num_keys,
                o.registration_processed
            FROM operators o
            WHERE o.chain_id = $1 AND o.owner_address = $2
            LIMIT 1
            "#,
        )
        .bind(self.chain_id)
        .bind(operator.as_slice())
        .fetch_optional(&self.pool)
        .await?;

        let Some(op_row) = operator_row else {
            return Ok(None);
        };

        let registration_root: Vec<u8> = op_row.get("registration_root");
        let owner_address: Vec<u8> = op_row.get("owner_address");
        let num_keys: Option<i32> = op_row.get("num_keys");
        let registration_processed: bool = op_row.get("registration_processed");

        let registration_root_b256 = B256::from_slice(&registration_root);

        // Get all events for this operator up to block_number
        // We join via operator_data which already has the registration_root indexed
        let event_rows = sqlx::query(
            r#"
            SELECT
                e.block_number,
                e.block_hash,
                e.tx_hash,
                e.tx_index,
                e.log_index,
                e.event_type,
                e.decoded_data,
                od.event_type as operator_event_type,
                od.deleted,
                od.equivocated,
                od.registered_at,
                od.slashed_at
            FROM operator_data od
            JOIN events e ON e.id = od.event_id
            JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
            WHERE od.registration_root = $1
                AND od.chain_id = $2
                AND b.number <= $3
                AND b.canonical = true
            ORDER BY e.block_number ASC, e.log_index ASC
            "#,
        )
        .bind(&registration_root)
        .bind(self.chain_id)
        .bind(block_number as i64)
        .fetch_all(&self.pool)
        .await?;

        // Get collateral total by summing all deltas up to the specified block
        // Some entries might have collateral_wei_total set (like registration), others just have delta
        let collateral_row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(oc.collateral_wei_delta), 0) +
                COALESCE(MAX(CASE WHEN oc.collateral_wei_total IS NOT NULL THEN oc.collateral_wei_total ELSE 0 END), 0)
                AS total_collateral
            FROM operator_collateral oc
            JOIN events e ON e.id = oc.event_id
            JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
            WHERE oc.registration_root = $1
                AND oc.chain_id = $2
                AND b.number <= $3
                AND b.canonical = true
            "#
        )
        .bind(&registration_root)
        .bind(self.chain_id)
        .bind(block_number as i64)
        .fetch_optional(&self.pool)
        .await?;

        let collateral_wei = if let Some(row) = collateral_row {
            let collateral_total: Option<sqlx::types::BigDecimal> = row.get("total_collateral");
            if let Some(total) = collateral_total {
                // Convert BigDecimal to U256
                if let Ok(val_str) = total.to_string().parse::<u128>() {
                    U256::from(val_str)
                } else {
                    U256::ZERO
                }
            } else {
                U256::ZERO
            }
        } else {
            U256::ZERO
        };

        // Build operator state from events
        let mut is_registered = false;
        let mut registered_at_block = None;
        let mut unregistered_at_block = None;
        let mut slashed_at_block = None;
        let mut deleted = false;
        let mut equivocated = false;
        let mut event_history = Vec::new();
        let mut commitment_protocols = Vec::new();
        let slash_history = Vec::new();

        for event_row in event_rows {
            let block_num: i64 = event_row.get("block_number");
            let block_hash: Vec<u8> = event_row.get("block_hash");
            let tx_hash: Vec<u8> = event_row.get("tx_hash");
            let tx_index: i32 = event_row.get("tx_index");
            let log_index: i32 = event_row.get("log_index");
            let event_type: String = event_row.get("event_type");
            let decoded_data: serde_json::Value = event_row.get("decoded_data");

            // Use operator_data columns for state tracking
            if let Ok(Some(reg_at)) = event_row.try_get::<Option<i64>, _>("registered_at") {
                if reg_at > 0 {
                    is_registered = true;
                    registered_at_block = Some(block_num as u64);
                }
            }
            if let Ok(Some(slash_at)) = event_row.try_get::<Option<i32>, _>("slashed_at") {
                if slash_at > 0 {
                    slashed_at_block = Some(slash_at as u64);
                }
            }
            if let Ok(Some(del)) = event_row.try_get::<Option<bool>, _>("deleted") {
                if del {
                    deleted = true;
                    is_registered = false;
                    unregistered_at_block = Some(block_num as u64);
                }
            }
            if let Ok(Some(equiv)) = event_row.try_get::<Option<bool>, _>("equivocated") {
                if equiv {
                    equivocated = true;
                }
            }

            let event_type_enum = match event_type.as_str() {
                "OperatorRegistered" => OperatorEventType::OperatorRegistered,
                "OperatorUnregistered" => OperatorEventType::OperatorUnregistered,
                "OperatorSlashed" => OperatorEventType::OperatorSlashed,
                "CollateralAdded" => OperatorEventType::CollateralAdded,
                "CollateralClaimed" => OperatorEventType::CollateralClaimed,
                "OperatorOptedIn" => OperatorEventType::OperatorOptedIn,
                "OperatorOptedOut" => OperatorEventType::OperatorOptedOut,
                _ => continue,
            };

            let operator_event = OperatorEvent {
                block_number: block_num as u64,
                block_hash: B256::from_slice(&block_hash),
                tx_hash: B256::from_slice(&tx_hash),
                tx_index: tx_index as u32,
                log_index: log_index as u32,
                event_type: event_type_enum,
                event_data: decoded_data.clone(),
                collateral_delta: None, // TODO: Calculate delta
            };

            event_history.push(operator_event);
        }

        // Get commitment protocols
        let commitment_rows = sqlx::query(
            r#"
            SELECT
                s.address as slasher_address,
                c.address as committer_address,
                osc.opted_in_at,
                osc.opted_out_at,
                osc.slashed
            FROM operator_slasher_commitment osc
            LEFT JOIN slasher s ON s.id = osc.slasher_id
            LEFT JOIN committer c ON c.id = osc.committer_id
            WHERE osc.registration_root = $1
                AND osc.chain_id = $2
            "#,
        )
        .bind(&registration_root)
        .bind(self.chain_id)
        .fetch_all(&self.pool)
        .await?;

        for commit_row in commitment_rows {
            let slasher_address: Vec<u8> = commit_row.get("slasher_address");
            let committer_address: Vec<u8> = commit_row.get("committer_address");
            let _opted_in_at: Option<chrono::DateTime<chrono::Utc>> = commit_row.get("opted_in_at");
            let _opted_out_at: Option<chrono::DateTime<chrono::Utc>> =
                commit_row.get("opted_out_at");
            let slashed: bool = commit_row.get("slashed");

            commitment_protocols.push(CommitmentProtocol {
                slasher_address: Address::from_slice(&slasher_address),
                committer_address: Address::from_slice(&committer_address),
                opted_in_at_block: None, // TODO: Convert timestamp to block number
                opted_out_at_block: None, // TODO: Convert timestamp to block number
                slashed,
            });
        }

        // TODO: Get slash history

        Ok(Some(OperatorState {
            operator_id: uuid::Uuid::new_v4(), // TODO: Get actual operator ID
            operator_address: Address::from_slice(&owner_address),
            registration_root: registration_root_b256,
            owner_address: Address::from_slice(&owner_address),
            num_keys: num_keys.map(|k| k as u32),
            is_registered,
            registration_processed,
            collateral_wei,
            registered_at_block,
            unregistered_at_block,
            slashed_at_block,
            deleted,
            equivocated,
            commitment_protocols,
            event_history,
            slash_history,
        }))
    }

    pub(super) async fn get_slashable_operators_impl(
        &self,
        block_number: u64,
    ) -> Result<Vec<urci_common::OperatorState>> {
        // Get all operators that:
        // 1. Are registered at this block
        // 2. Have not been unregistered (or unregistration delay hasn't passed)
        // 3. Have collateral > 0
        // 4. Are past the fraud proof window (if applicable)

        let all_operators = self.get_operators_at_block_impl(block_number).await?;

        let slashable: Vec<_> = all_operators
            .into_iter()
            .filter(|op| {
                // Must be registered
                if !op.is_registered {
                    return false;
                }

                // Must have collateral
                if op.collateral_wei.is_zero() {
                    return false;
                }

                // Must not be deleted
                if op.deleted {
                    return false;
                }

                // If unregistered, check if still in slashable period
                // (This would require config values - for now allow if not deleted)

                true
            })
            .collect();

        Ok(slashable)
    }

    /// Get timeline of all operator events
    #[allow(dead_code)]
    pub(super) async fn get_operator_timeline_impl(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Vec<urci_common::api::OperatorTimelineEvent>> {
        use sqlx::Row;
        use urci_common::api::OperatorTimelineEvent;

        let query = r#"
            SELECT
                e.event_type::text as event_type,
                e.block_number,
                b.timestamp,
                e.decoded_data
            FROM operator_data od
            JOIN events e ON e.id = od.event_id
            JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
            WHERE od.chain_id = $1
              AND od.registration_root = $2
            ORDER BY e.block_number ASC, e.log_index ASC
        "#;

        let rows = sqlx::query(query)
            .bind(chain_id)
            .bind(registration_root.as_slice())
            .fetch_all(&self.pool)
            .await?;

        let timeline = rows
            .into_iter()
            .map(|row| {
                let event_type: String = row.get("event_type");
                let block_number: i64 = row.get("block_number");
                let timestamp: Option<i64> = row.get("timestamp");
                let decoded_data: serde_json::Value = row.get("decoded_data");

                OperatorTimelineEvent {
                    event_type,
                    block_number,
                    timestamp: timestamp.map(|t| t.to_string()),
                    data: decoded_data,
                }
            })
            .collect();

        Ok(timeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_helpers::{setup_test_db, TestFixture};
    use crate::traits::Db;
    use alloy_primitives::address;

    #[tokio::test]
    async fn test_get_operator_state_impl_complete_lifecycle() {
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

        // Create a complete test fixture with all event types
        let fixture = TestFixture::new()
            .with_registration()
            .with_collateral()
            .with_commitment_opt_in()
            .with_commitment_opt_out()
            .with_collateral_claimed();

        // Write all blocks
        fixture
            .write_to_adapter(&mut adapter)
            .await
            .expect("Failed to write blocks");

        // Test 1: Get state at block 1 (just registration)
        let state_at_1 = adapter
            .get_operator_state_impl(&fixture.owner_address, 1)
            .await
            .expect("Failed to get operator state at block 1")
            .expect("Operator should exist at block 1");

        assert!(state_at_1.is_registered, "Should be registered at block 1");
        assert_eq!(state_at_1.owner_address, fixture.owner_address);
        assert_eq!(state_at_1.registration_root, fixture.registration_root);
        assert!(
            state_at_1.collateral_wei > alloy_primitives::U256::ZERO,
            "Should have initial collateral"
        );
        assert_eq!(state_at_1.event_history.len(), 1, "Should have 1 event");

        // Test 2: Get state at block 2 (after collateral added)
        let state_at_2 = adapter
            .get_operator_state_impl(&fixture.owner_address, 2)
            .await
            .expect("Failed to get operator state at block 2")
            .expect("Operator should exist at block 2");

        assert!(
            state_at_2.collateral_wei > state_at_1.collateral_wei,
            "Collateral should increase after block 2"
        );
        assert_eq!(state_at_2.event_history.len(), 2, "Should have 2 events");

        // Test 3: Get state at block 5 (complete lifecycle)
        let state_at_5 = adapter
            .get_operator_state_impl(&fixture.owner_address, 5)
            .await
            .expect("Failed to get operator state at block 5")
            .expect("Operator should exist at block 5");

        assert_eq!(state_at_5.event_history.len(), 5, "Should have 5 events");
        assert!(
            !state_at_5.commitment_protocols.is_empty(),
            "Should have commitment protocols"
        );
    }

    #[tokio::test]
    async fn test_get_operators_at_block_impl() {
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

        // Create two operators with different block offsets to avoid duplicate keys
        let fixture1 = TestFixture::new().with_registration().with_collateral();
        let mut fixture2 = TestFixture::new().with_offset(100); // Start at block 101
        fixture2.registration_root = alloy_primitives::B256::from([2; 32]);
        fixture2.owner_address = alloy_primitives::Address::from([2; 20]);
        fixture2 = fixture2.with_registration().with_collateral();

        // Write both operators
        fixture1
            .write_to_adapter(&mut adapter)
            .await
            .expect("Failed to write fixture1");
        fixture2
            .write_to_adapter(&mut adapter)
            .await
            .expect("Failed to write fixture2");

        // Get all operators at block 102 (after both are registered)
        let operators_at_102 = adapter
            .get_operators_at_block_impl(102)
            .await
            .expect("Failed to get operators at block 102");

        assert_eq!(
            operators_at_102.len(),
            2,
            "Should have 2 operators at block 102"
        );

        // Verify both operators are present
        let has_operator1 = operators_at_102
            .iter()
            .any(|op| op.owner_address == fixture1.owner_address);
        let has_operator2 = operators_at_102
            .iter()
            .any(|op| op.owner_address == fixture2.owner_address);

        assert!(has_operator1, "Should have operator 1");
        assert!(has_operator2, "Should have operator 2");
    }

    #[tokio::test]
    async fn test_get_slashable_operators_impl() {
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

        // Create operator with collateral (slashable)
        let fixture_slashable = TestFixture::new().with_registration().with_collateral();

        // Create operator with no collateral (not slashable) with different block offset
        let mut fixture_no_collateral = TestFixture::new().with_offset(100);
        fixture_no_collateral.registration_root = alloy_primitives::B256::from([3; 32]);
        fixture_no_collateral.owner_address = alloy_primitives::Address::from([3; 20]);
        fixture_no_collateral = fixture_no_collateral.with_registration(); // No collateral added

        fixture_slashable
            .write_to_adapter(&mut adapter)
            .await
            .expect("Failed to write slashable");
        fixture_no_collateral
            .write_to_adapter(&mut adapter)
            .await
            .expect("Failed to write no collateral");

        // Get slashable operators
        let slashable = adapter
            .get_slashable_operators_impl(2)
            .await
            .expect("Failed to get slashable operators");

        assert_eq!(slashable.len(), 1, "Should have 1 slashable operator");
        assert_eq!(slashable[0].owner_address, fixture_slashable.owner_address);
        assert!(slashable[0].collateral_wei > alloy_primitives::U256::ZERO);
    }

    #[tokio::test]
    async fn test_get_operator_state_with_slashing() {
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

        // Create operator with registration, collateral, commitment, and slashing
        let fixture = TestFixture::new()
            .with_registration()
            .with_collateral()
            .with_commitment_opt_in()
            .with_slashing();

        fixture
            .write_to_adapter(&mut adapter)
            .await
            .expect("Failed to write fixture");

        // Get state after slashing
        let state_after_slash = adapter
            .get_operator_state_impl(&fixture.owner_address, 4)
            .await
            .expect("Failed to get operator state")
            .expect("Operator should exist");

        assert_eq!(
            state_after_slash.slashed_at_block,
            Some(4),
            "Should be slashed at block 4"
        );
        assert_eq!(
            state_after_slash.event_history.len(),
            4,
            "Should have 4 events"
        );

        // Find the slashing event
        let has_slash_event = state_after_slash.event_history.iter().any(|e| {
            matches!(
                e.event_type,
                urci_common::OperatorEventType::OperatorSlashed
            )
        });

        assert!(has_slash_event, "Should have slashing event in history");
    }

    #[tokio::test]
    async fn test_get_operator_state_nonexistent() {
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

        let nonexistent_address = alloy_primitives::Address::from([99; 20]);

        let result = adapter
            .get_operator_state_impl(&nonexistent_address, 100)
            .await
            .expect("Query should succeed");

        assert!(
            result.is_none(),
            "Should return None for nonexistent operator"
        );
    }

    #[tokio::test]
    async fn test_get_operators_at_block_before_registration() {
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

        // Create operator registered at block 5
        let fixture = TestFixture::new().with_registration();
        fixture
            .write_to_adapter(&mut adapter)
            .await
            .expect("Failed to write fixture");

        // Query at block 0 (before registration)
        let operators_at_0 = adapter
            .get_operators_at_block_impl(0)
            .await
            .expect("Failed to get operators at block 0");

        assert_eq!(
            operators_at_0.len(),
            0,
            "Should have no operators before registration"
        );
    }
}
