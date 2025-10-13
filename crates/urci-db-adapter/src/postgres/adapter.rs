use alloy_primitives::Address;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// PostgreSQL adapter for URC indexer
#[derive(Clone)]
pub struct PostgresAdapter {
    pub(crate) pool: Pool<Postgres>,
    pub(crate) writer_id: Option<Uuid>,
    pub(crate) session_id: Option<String>,
    pub(crate) db_url: String,
    pub(crate) registry_address: Option<Address>,
    pub(crate) chain_id: i64,  // Required chain_id
    pub failed_state: std::sync::Arc<std::sync::atomic::AtomicBool>,  // Once true, all writes go to error table
}

impl PostgresAdapter {
    /// Create a new PostgreSQL adapter with a specific chain_id
    /// Note: You must call connect() or initialize() to establish a database connection
    pub fn new(_chain_id: i64) -> Self {
        // Can't create a pool without a connection string, so we'll initialize it in connect()
        // This is a breaking change from the original design
        panic!("PostgresAdapter::new() cannot create a pool without a database URL. Use PostgresAdapter::connect_new() instead.");
    }

    /// Create and connect to a PostgreSQL adapter
    pub async fn connect_new(chain_id: i64, db_url: &str) -> Result<Self, sqlx::Error> {
        use sqlx::postgres::PgPoolOptions;

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(db_url)
            .await?;

        Ok(Self {
            pool,
            writer_id: None,
            session_id: None,
            db_url: db_url.to_string(),
            registry_address: None,
            chain_id,
            failed_state: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Get the writer ID
    pub fn writer_id(&self) -> Option<Uuid> {
        self.writer_id
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.chain_id as u64
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}