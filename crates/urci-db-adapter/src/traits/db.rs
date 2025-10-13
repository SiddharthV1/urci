use alloy_primitives::Address;
use async_trait::async_trait;
use eyre::Result;
use urci_common::WorkId;

use crate::state::DBState;

#[async_trait]
pub trait Db: Send + Sync {
    async fn connect(&mut self, db_url: &str) -> Result<(uuid::Uuid, Option<u64>, Option<WorkId>)>;

    async fn is_initialized(&self) -> Result<bool>;

    async fn run_migrations(&mut self) -> Result<()>;

    async fn create_session(&mut self, registry_address: Address, start_block: Option<u64>) -> Result<uuid::Uuid>;

    async fn write_config(&mut self, writer_id: uuid::Uuid, registry_address: alloy_primitives::Address, config: urci_common::IRegistry::Config) -> Result<()>;

    async fn get_indexer_height(&self) -> Result<(uuid::Uuid, Option<u64>, Option<WorkId>)>;

    async fn get_db_state(&self) -> Result<DBState>;

    /// Initial database setup - creates tables, runs migrations, establishes writer session
    async fn initialize(
        &mut self,
        registry_address: Address,
        start_block: u64,
    ) -> Result<(uuid::Uuid, Option<u64>, Option<WorkId>)>;
}