use alloy_primitives::{hex, Address, B256};
use eyre::Result;
use sqlx::Row;

use super::super::adapter::PostgresAdapter;
use crate::sql_constants;

impl PostgresAdapter {
    pub(super) async fn get_operator_commitments_impl(
        &self,
        chain_id: i64,
        registration_root: &B256,
        limit: i64,
        _cursor: Option<String>,
    ) -> Result<Vec<urci_common::api::OperatorCommitmentResponse>> {
        use urci_common::api::OperatorCommitmentResponse;

        let rows = sqlx::query(sql_constants::SELECT_OPERATOR_COMMITMENTS)
            .bind(registration_root.as_slice())
            .bind(chain_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let commitments = rows
            .into_iter()
            .map(|row| {
                let slasher_address: Option<Vec<u8>> = row.get("slasher_address");
                let committer_address: Option<Vec<u8>> = row.get("committer_address");
                let opted_in_at: Option<chrono::DateTime<chrono::Utc>> = row.get("opted_in_at");
                let opted_out_at: Option<chrono::DateTime<chrono::Utc>> = row.get("opted_out_at");
                let slashed: Option<bool> = row.get("slashed");

                // Determine status based on state
                let status = if slashed.unwrap_or(false) {
                    "slashed".to_string()
                } else if opted_out_at.is_some() {
                    "opted_out".to_string()
                } else {
                    "active".to_string()
                };

                OperatorCommitmentResponse {
                    slasher_address: slasher_address
                        .map(|addr| format!("0x{}", hex::encode(addr)))
                        .unwrap_or_else(|| "0x".to_string()),
                    committer_address: committer_address
                        .map(|addr| format!("0x{}", hex::encode(addr))),
                    opted_in_at: opted_in_at.map(|dt| dt.to_rfc3339()),
                    opted_out_at: opted_out_at.map(|dt| dt.to_rfc3339()),
                    status,
                    event_id: None, // TODO: Add event_id column to operator_slasher_commitment table
                }
            })
            .collect();

        Ok(commitments)
    }

    pub(super) async fn list_slashers_impl(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::SlasherResponse>> {
        use urci_common::api::SlasherResponse;

        let rows = sqlx::query(sql_constants::SELECT_SLASHERS)
            .bind(chain_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let slashers = rows
            .into_iter()
            .map(|row| {
                let address: Vec<u8> = row.get("address");
                let chain_id: i32 = row.get("chain_id"); // Schema has INTEGER (INT4)
                let event_id: Option<i64> = row.get("event_id");
                let created_at: Option<chrono::DateTime<chrono::Utc>> = row.get("created_at");

                SlasherResponse {
                    address: Address::from_slice(&address).to_string(),
                    chain_id: chain_id as i64,
                    event_id,
                    created_at: created_at.map(|dt| dt.to_string()),
                }
            })
            .collect();

        Ok(slashers)
    }

    pub(super) async fn get_slasher_operators_impl(
        &self,
        chain_id: i64,
        slasher_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorWithCommitmentResponse>> {
        use urci_common::api::{OperatorResponse, OperatorWithCommitmentResponse};

        let rows = sqlx::query(sql_constants::SELECT_SLASHER_OPERATORS)
            .bind(chain_id)
            .bind(slasher_address.as_slice())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let operators = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;

                // Extract all fields manually (PgRow doesn't implement Clone)
                let registration_root: Vec<u8> = row.get("registration_root");
                let chain_id: i64 = row.get("chain_id");
                let owner_address: Vec<u8> = row.get("owner_address");
                let num_keys: Option<i32> = row.get("num_keys");
                let registration_processed: bool = row.get("registration_processed");
                let block_number: Option<i64> = row.get("block_number");
                let collateral_wei_total: Option<sqlx::types::BigDecimal> =
                    row.get("collateral_wei_total");
                let last_event_type: Option<String> = row.get("last_event_type");
                let deleted: Option<bool> = row.get("deleted");
                let equivocated: Option<bool> = row.get("equivocated");
                let canonical: Option<bool> = row.get("canonical");
                let finalized: Option<bool> = row.get("finalized");
                let sync_status: Option<String> = row.get("sync_status");

                // Commitment fields
                let opted_in_at: Option<chrono::DateTime<chrono::Utc>> = row.get("opted_in_at");
                let opted_out_at: Option<chrono::DateTime<chrono::Utc>> = row.get("opted_out_at");
                let slashed: bool = row.get("slashed");
                let committer_address: Option<Vec<u8>> = row.get("committer_address");
                let commitment_status: String = row.get("commitment_status");

                OperatorWithCommitmentResponse {
                    operator: OperatorResponse {
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
                    },
                    commitment_status,
                    opted_in_at: opted_in_at.map(|dt| dt.to_string()),
                    opted_out_at: opted_out_at.map(|dt| dt.to_string()),
                    committer_address: committer_address
                        .map(|addr| Address::from_slice(&addr).to_string()),
                    slashed,
                }
            })
            .collect();

        Ok(operators)
    }

    pub(super) async fn get_slasher_stats_impl(
        &self,
        chain_id: i64,
        slasher_address: &Address,
    ) -> Result<Option<urci_common::api::SlasherStatsResponse>> {
        use urci_common::api::{SlasherStatsResponse, SlashesByType};

        let row = sqlx::query(sql_constants::SELECT_SLASHER_STATS)
            .bind(chain_id)
            .bind(slasher_address.as_slice())
            .fetch_optional(&self.pool)
            .await?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let slasher_address_bytes: Vec<u8> = row.get("slasher_address");
        let chain_id: i32 = row.get("chain_id"); // Schema has INTEGER (INT4)
        let total_operators: i64 = row.get("total_operators");
        let active_operators: i64 = row.get("active_operators");
        let total_slashes: i64 = row.get("total_slashes");
        let registration_slashes: i64 = row.get("registration_slashes");
        let commitment_slashes: i64 = row.get("commitment_slashes");
        let offchain_slashes: i64 = row.get("offchain_slashes");
        let equivocation_slashes: i64 = row.get("equivocation_slashes");
        let total_rewards: sqlx::types::BigDecimal = row.get("total_rewards");
        let total_burned: sqlx::types::BigDecimal = row.get("total_burned");

        Ok(Some(SlasherStatsResponse {
            slasher_address: Address::from_slice(&slasher_address_bytes).to_string(),
            chain_id: chain_id as i64,
            total_operators,
            active_operators,
            total_slashes,
            slashes_by_type: SlashesByType {
                registration: registration_slashes,
                commitment: commitment_slashes,
                offchain_delegation: offchain_slashes,
                equivocation: equivocation_slashes,
            },
            total_rewards_wei: total_rewards.to_string(),
            total_burned_wei: total_burned.to_string(),
        }))
    }

    pub(super) async fn list_committers_impl(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::CommitterResponse>> {
        use urci_common::api::CommitterResponse;

        let rows = sqlx::query(sql_constants::SELECT_COMMITTERS)
            .bind(chain_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let committers = rows
            .into_iter()
            .map(|row| {
                let address: Vec<u8> = row.get("address");
                let chain_id: i32 = row.get("chain_id"); // Schema has INTEGER (INT4)
                let event_id: Option<i64> = row.get("event_id");
                let created_at: Option<chrono::DateTime<chrono::Utc>> = row.get("created_at");

                CommitterResponse {
                    address: Address::from_slice(&address).to_string(),
                    chain_id: chain_id as i64,
                    event_id,
                    created_at: created_at.map(|dt| dt.to_string()),
                }
            })
            .collect();

        Ok(committers)
    }

    pub(super) async fn get_committer_operators_impl(
        &self,
        chain_id: i64,
        committer_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorWithCommitmentResponse>> {
        use urci_common::api::{OperatorResponse, OperatorWithCommitmentResponse};

        let rows = sqlx::query(sql_constants::SELECT_COMMITTER_OPERATORS)
            .bind(chain_id)
            .bind(committer_address.as_slice())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let operators = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;

                // Extract all fields manually (PgRow doesn't implement Clone)
                let registration_root: Vec<u8> = row.get("registration_root");
                let chain_id: i64 = row.get("chain_id");
                let owner_address: Vec<u8> = row.get("owner_address");
                let num_keys: Option<i32> = row.get("num_keys");
                let registration_processed: bool = row.get("registration_processed");
                let block_number: Option<i64> = row.get("block_number");
                let collateral_wei_total: Option<sqlx::types::BigDecimal> =
                    row.get("collateral_wei_total");
                let last_event_type: Option<String> = row.get("last_event_type");
                let deleted: Option<bool> = row.get("deleted");
                let equivocated: Option<bool> = row.get("equivocated");
                let canonical: Option<bool> = row.get("canonical");
                let finalized: Option<bool> = row.get("finalized");
                let sync_status: Option<String> = row.get("sync_status");

                // Commitment fields
                let opted_in_at: Option<chrono::DateTime<chrono::Utc>> = row.get("opted_in_at");
                let opted_out_at: Option<chrono::DateTime<chrono::Utc>> = row.get("opted_out_at");
                let slashed: bool = row.get("slashed");
                let _slasher_address: Option<Vec<u8>> = row.get("slasher_address");
                let commitment_status: String = row.get("commitment_status");

                OperatorWithCommitmentResponse {
                    operator: OperatorResponse {
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
                    },
                    commitment_status,
                    opted_in_at: opted_in_at.map(|dt| dt.to_string()),
                    opted_out_at: opted_out_at.map(|dt| dt.to_string()),
                    committer_address: Some(committer_address.to_string()),
                    slashed,
                }
            })
            .collect();

        Ok(operators)
    }
}
