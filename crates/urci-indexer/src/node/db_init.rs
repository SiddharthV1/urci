//! Database initialization logic

use alloy_primitives::Address;
use eyre::Result;
use tracing::info;
use urci_db_adapter::{Db, PostgresAdapter};
use uuid::Uuid;

/// Writer ID returned from database initialization
pub type WriterId = Uuid;

/// Initialize database connection, run migrations if needed, and create session
///
/// Returns (adapter, writer_id) tuple
pub async fn initialize_database(
    chain_id: i64,
    database_url: &str,
    registry_address: Address,
    backfill_from: Option<u64>,
) -> Result<(PostgresAdapter, WriterId)> {
    info!("Initializing database connection");

    let mut db_adapter = PostgresAdapter::connect_new(chain_id, database_url).await?;
    let tables_exist = db_adapter.is_initialized().await?;

    if !tables_exist {
        info!("Running database migrations");
        db_adapter.run_migrations().await?;
    } else {
        info!("Database tables already exist, skipping migrations");
    }

    // Always create session - chain_roots INSERT has ON CONFLICT DO NOTHING for idempotency
    db_adapter
        .create_session(registry_address, backfill_from)
        .await?;

    let (writer_id, _start_block, _parent_work_id) = db_adapter.get_indexer_height().await?;
    info!("Database initialized, writer_id: {}", writer_id);

    Ok((db_adapter, writer_id))
}
