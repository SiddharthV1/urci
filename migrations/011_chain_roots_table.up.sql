-- =============================================================================
-- CHAIN ROOTS TABLE
-- =============================================================================

-- Chain roots configuration
CREATE TABLE chain_roots (
    chain_id BIGINT PRIMARY KEY,
    start_height BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_positive_height CHECK (start_height >= 0)
);

SELECT trigger_updated_at('chain_roots');
