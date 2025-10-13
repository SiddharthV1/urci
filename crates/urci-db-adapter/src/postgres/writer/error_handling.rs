use eyre::Result;
use sqlx::Transaction;
use tracing::error;
use urci_common::UrciBlockUpdate;

use crate::postgres::adapter::PostgresAdapter;
use crate::sql_constants;

// Helper to classify database errors
// Uses both sqlx::error::ErrorKind and PostgreSQL error codes
// See: https://www.postgresql.org/docs/current/errcodes-appendix.html
pub(super) fn is_retryable_error(err: &eyre::Report) -> bool {
    use sqlx::error::ErrorKind;

    // Try to downcast to sqlx::Error
    if let Some(sqlx_err) = err.downcast_ref::<sqlx::Error>() {
        match sqlx_err {
            // Connection/timeout errors - retryable
            sqlx::Error::PoolTimedOut => true,
            sqlx::Error::PoolClosed => true,
            sqlx::Error::Io(_) => true,

            // Database errors - use kind() enum AND error code for precise classification
            sqlx::Error::Database(db_err) => {
                // First check the ErrorKind enum
                match db_err.kind() {
                    // These kinds are clearly NOT retryable (data errors)
                    ErrorKind::UniqueViolation => false,
                    ErrorKind::ForeignKeyViolation => false,
                    ErrorKind::NotNullViolation => false,
                    ErrorKind::CheckViolation => false,

                    // Other kinds - check PostgreSQL error code for retryability
                    _ => {
                        let code_str = db_err.code().map(|c| c.to_string());
                        if let Some(code) = code_str.as_deref() {
                            matches!(
                                code,
                                // Class 40 — Transaction Rollback (retryable)
                                "40001" |  // serialization_failure
                                "40P01" |  // deadlock_detected

                                // Class 55 — Object Not In Prerequisite State (retryable)
                                "55P03" |  // lock_not_available

                                // Class 08 — Connection Exception (retryable)
                                "08000" |  // connection_exception
                                "08003" |  // connection_does_not_exist
                                "08006" |  // connection_failure
                                "08P01" // protocol_violation
                            )
                        } else {
                            // No error code - treat as non-retryable
                            false
                        }
                    }
                }
            }

            // Everything else is a data error - NOT retryable
            _ => false,
        }
    } else {
        // Not a sqlx::Error - treat as non-retryable
        false
    }
}

impl PostgresAdapter {
    /// Write failed block to error table
    pub(super) async fn write_to_error_table(
        &self,
        block: &UrciBlockUpdate,
        error_message: &str,
        error_context: Option<serde_json::Value>,
    ) -> Result<()> {
        error!(
            block_number = block.block_number,
            block_hash = %block.block_hash,
            error = error_message,
            "Writing failed block to error table"
        );

        // Serialize block and events to JSON
        let block_data = serde_json::to_value(block)?;
        let events_data = serde_json::to_value(&block.events)?;

        sqlx::query(
            "INSERT INTO failed_block_writes (
                chain_id, block_number, block_hash,
                block_data, events_data,
                error_message, error_location, error_context,
                writer_id, session_id, status, retry_count
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'requires_healing', 0)
            ON CONFLICT DO NOTHING",
        )
        .bind(self.chain_id)
        .bind(block.block_number as i64)
        .bind(block.block_hash.as_slice())
        .bind(block_data)
        .bind(events_data)
        .bind(error_message)
        .bind("write_blocks") // error_location
        .bind(error_context)
        .bind(self.writer_id)
        .bind(self.session_id.as_deref())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Helper: Take advisory lock within a transaction using chain_id as key
    pub(super) async fn acquire_tx_lock(
        &self,
        tx: &mut Transaction<'static, sqlx::Postgres>,
    ) -> Result<()> {
        sqlx::query(sql_constants::TRY_ADVISORY_LOCK)
            .bind(self.chain_id)
            .execute(tx.as_mut())
            .await?;
        Ok(())
    }
}
