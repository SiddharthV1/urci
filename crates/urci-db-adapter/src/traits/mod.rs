//! Database adapter traits module

pub mod db;
pub mod reader;
pub mod writer;

pub use db::Db;
pub use reader::AdapterReader;
pub use writer::AdapterWriter;