//! Pagination response types

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// Pagination metadata for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct PaginationMeta {
    // Cursor for next page (null if no more results)
    #[cfg_attr(feature = "utoipa", schema(nullable = true))]
    pub cursor: Option<String>,

    // Number of results requested
    pub limit: i64,

    // Whether there are more results available
    pub has_more: bool,
}
