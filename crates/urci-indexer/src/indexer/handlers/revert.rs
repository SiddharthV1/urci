//! Revert handler - processes chain reverts

use alloy_primitives::B256;
use eyre::Result;
use tracing::info;
use urci_db_adapter::{AdapterWriter, PostgresAdapter};

/// Handle Revert update
///
/// A revert is like a reorg but with no new chain - just mark blocks as non-canonical
pub async fn handle_revert(
    from: (u64, B256),
    to: (u64, B256),
    adapter: &mut PostgresAdapter,
) -> Result<()> {
    info!(
        "Indexer handling revert: marking blocks {} to {} as non-canonical",
        from.0, to.0
    );

    // Revert is like reorg but with no new chain - just mark from..=to as non-canonical
    adapter.handle_reorg(from.0, to.0).await?;

    info!("✓ Revert handled successfully");

    Ok(())
}
