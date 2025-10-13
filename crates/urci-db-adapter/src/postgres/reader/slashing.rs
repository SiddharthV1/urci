use alloy_primitives::{Address, B256};
use eyre::Result;
use sqlx::Row;

use super::super::adapter::PostgresAdapter;
use crate::sql_constants;
use urci_common::api::{OperatorSlashingResponse, SlashEventResponse};

impl PostgresAdapter {
    pub(super) async fn get_operator_slashing_impl(
        &self,
        chain_id: i64,
        registration_root: &B256,
        limit: i64,
        _cursor: Option<String>,
    ) -> Result<Option<OperatorSlashingResponse>> {
        let rows = sqlx::query(sql_constants::SELECT_OPERATOR_SLASHING)
            .bind(registration_root.as_slice())
            .bind(chain_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let mut total_burned = 0i64;
        let slashes: Vec<SlashEventResponse> = rows
            .into_iter()
            .map(|row| {
                let slash_type: String = row.get("slash_type");
                let event_id: Option<i64> = row.get("event_id");
                let block_number: i64 = row.get("block_number");
                let sender_address: Option<Vec<u8>> = row.get("sender_address");
                let sender_reward_wei: Option<sqlx::types::BigDecimal> =
                    row.get("sender_reward_wei");
                let burned_wei: Option<sqlx::types::BigDecimal> = row.get("burned_wei");
                let created_at: Option<chrono::DateTime<chrono::Utc>> = row.get("created_at");

                // Accumulate total burned
                if let Some(ref burned) = burned_wei {
                    if let Ok(burned_i64) = burned.to_string().parse::<i64>() {
                        total_burned += burned_i64;
                    }
                }

                SlashEventResponse {
                    slash_type,
                    event_id,
                    block_number,
                    sender_address: sender_address
                        .map(|addr| format!("0x{}", alloy_primitives::hex::encode(addr)))
                        .unwrap_or_else(|| "0x".to_string()),
                    sender_reward_wei: sender_reward_wei
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "0".to_string()),
                    burned_wei: burned_wei
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "0".to_string()),
                    timestamp: created_at.map(|dt| dt.to_rfc3339()),
                }
            })
            .collect();

        let total_slashes = slashes.len() as i64;

        Ok(Some(OperatorSlashingResponse {
            total_slashes,
            total_burned_wei: total_burned.to_string(),
            slashes,
        }))
    }

    pub(super) async fn get_slasher_slashing_events_impl(
        &self,
        chain_id: i64,
        slasher_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::SlashingEventResponse>> {
        use urci_common::api::SlashingEventResponse;

        let rows = sqlx::query(sql_constants::SELECT_SLASHER_SLASHING_EVENTS)
            .bind(chain_id)
            .bind(slasher_address.as_slice())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let events = rows
            .into_iter()
            .map(|row| {
                let slash_type: String = row.get("slash_type");
                let event_id: Option<i64> = row.get("event_id");
                let block_number: i64 = row.get("block_number");
                let registration_root: Vec<u8> = row.get("registration_root");
                let sender_address: Vec<u8> = row.get("sender_address");
                let sender_reward_wei: Option<sqlx::types::BigDecimal> =
                    row.get("sender_reward_wei");
                let burned_wei: Option<sqlx::types::BigDecimal> = row.get("burned_wei");
                let timestamp: Option<i64> = row.get("timestamp");
                let details: Option<serde_json::Value> = row.get("details");

                SlashingEventResponse {
                    slash_type,
                    event_id,
                    block_number,
                    registration_root: B256::from_slice(&registration_root).to_string(),
                    sender_address: Address::from_slice(&sender_address).to_string(),
                    sender_reward_wei: sender_reward_wei
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "0".to_string()),
                    burned_wei: burned_wei
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "0".to_string()),
                    timestamp: timestamp
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        .map(|dt| dt.to_string()),
                    details: details.unwrap_or(serde_json::Value::Null),
                }
            })
            .collect();

        Ok(events)
    }

    pub(super) async fn get_committer_slashing_events_impl(
        &self,
        chain_id: i64,
        committer_address: &Address,
        limit: i64,
    ) -> Result<Vec<urci_common::api::CommitterSlashingEventResponse>> {
        use urci_common::api::CommitterSlashingEventResponse;

        let rows = sqlx::query(sql_constants::SELECT_COMMITTER_SLASHING_EVENTS)
            .bind(chain_id)
            .bind(committer_address.as_slice())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let events = rows
            .into_iter()
            .map(|row| {
                let slash_type: String = row.get("slash_type");
                let event_id: Option<i64> = row.get("event_id");
                let block_number: i64 = row.get("block_number");
                let registration_root: Vec<u8> = row.get("registration_root");
                let sender_address: Vec<u8> = row.get("sender_address");
                let committer_address: Vec<u8> = row.get("committer_address");
                let sender_reward_wei: Option<sqlx::types::BigDecimal> =
                    row.get("sender_reward_wei");
                let burned_wei: Option<sqlx::types::BigDecimal> = row.get("burned_wei");
                let timestamp: Option<i64> = row.get("timestamp");
                let details: Option<serde_json::Value> = row.get("details");

                CommitterSlashingEventResponse {
                    slash_type,
                    event_id,
                    block_number,
                    registration_root: B256::from_slice(&registration_root).to_string(),
                    sender_address: Address::from_slice(&sender_address).to_string(),
                    committer_address: Address::from_slice(&committer_address).to_string(),
                    sender_reward_wei: sender_reward_wei
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "0".to_string()),
                    burned_wei: burned_wei
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "0".to_string()),
                    timestamp: timestamp
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        .map(|dt| dt.to_string()),
                    details: details.unwrap_or(serde_json::Value::Null),
                }
            })
            .collect();

        Ok(events)
    }
}
