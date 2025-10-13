//! PostgreSQL adapter implementation

pub mod adapter;
pub mod db;
pub mod reader;
pub mod writer;

// Export test helpers for use in integration tests and other crates
pub mod test_helpers;

pub use adapter::PostgresAdapter;