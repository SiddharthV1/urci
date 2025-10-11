-- =============================================================================
-- CHAIN TABLE
-- =============================================================================

CREATE TABLE chain (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

SELECT trigger_updated_at('chain');
