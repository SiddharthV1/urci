//! Collateral management queries

use alloy_primitives::B256;
use eyre::Result;
use sqlx::Row;

use crate::sql_constants;
use super::super::adapter::PostgresAdapter;

impl PostgresAdapter {
    pub(super) async fn get_operator_collateral_impl(
        &self,
        chain_id: i64,
        registration_root: &B256,
    ) -> Result<Option<urci_common::api::OperatorCollateralResponse>> {
        use urci_common::api::{CollateralEntry, OperatorCollateralResponse};

        let rows = sqlx::query(sql_constants::SELECT_OPERATOR_COLLATERAL_HISTORY)
            .bind(registration_root.as_slice())
            .bind(chain_id)
            .fetch_all(&self.pool)
            .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let mut history: Vec<CollateralEntry> = Vec::new();
        let mut current_total = "0".to_string();

        for row in rows {
            let collateral_wei_total: Option<sqlx::types::BigDecimal> = row.get("collateral_wei_total");
            let collateral_wei_delta: Option<sqlx::types::BigDecimal> = row.get("collateral_wei_delta");
            let event_id: Option<i64> = row.get("event_id");
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.get("created_at");

            // Update current total if present
            if let Some(total) = &collateral_wei_total {
                current_total = total.to_string();
            }

            history.push(CollateralEntry {
                collateral_wei_total: collateral_wei_total
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "0".to_string()),
                collateral_wei_delta: collateral_wei_delta.map(|v| v.to_string()),
                event_id,
                created_at: created_at.map(|dt| dt.to_string()),
            });
        }

        Ok(Some(OperatorCollateralResponse {
            current_total,
            history,
        }))
    }

    pub(super) async fn get_total_collateral_impl(&self, chain_id: i64) -> Result<Option<urci_common::api::TotalCollateralResponse>> {
        use urci_common::api::TotalCollateralResponse;

        let row = sqlx::query(sql_constants::SELECT_TOTAL_COLLATERAL)
            .bind(chain_id)
            .fetch_optional(&self.pool)
            .await?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let chain_id: i64 = row.get("chain_id");
        let total_collateral_wei: sqlx::types::BigDecimal = row.get("total_collateral_wei");
        let total_operators: i64 = row.get("total_operators");
        let operators_with_collateral: i64 = row.get("operators_with_collateral");
        let average_collateral_wei: sqlx::types::BigDecimal = row.get("average_collateral_wei");

        Ok(Some(TotalCollateralResponse {
            chain_id,
            total_collateral_wei: total_collateral_wei.to_string(),
            total_operators,
            operators_with_collateral,
            average_collateral_wei: average_collateral_wei.to_string(),
        }))
    }

    pub(super) async fn get_operators_by_collateral_impl(
        &self,
        chain_id: i64,
        min_collateral: Option<String>,
        max_collateral: Option<String>,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorResponse>> {
        let min = min_collateral.unwrap_or_else(|| "0".to_string());
        let max = max_collateral.clone();

        let mut query = sqlx::query(sql_constants::SELECT_OPERATORS_BY_COLLATERAL)
            .bind(chain_id)
            .bind(&min);

        // Bind max_collateral as Option
        if let Some(max_val) = max {
            query = query.bind(Some(max_val));
        } else {
            query = query.bind(None::<String>);
        }

        query = query.bind(limit);

        let rows = query.fetch_all(&self.pool).await?;

        let operators = rows
            .into_iter()
            .map(|row| self.map_operator_row(row))
            .collect();

        Ok(operators)
    }

    pub(super) async fn get_top_operators_by_collateral_impl(
        &self,
        chain_id: i64,
        limit: i64,
    ) -> Result<Vec<urci_common::api::OperatorResponse>> {
        let rows = sqlx::query(sql_constants::SELECT_TOP_OPERATORS_BY_COLLATERAL)
            .bind(chain_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let operators = rows
            .into_iter()
            .map(|row| self.map_operator_row(row))
            .collect();

        Ok(operators)
    }
}
