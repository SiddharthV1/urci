-- =============================================================================
-- OPERATOR FORK VIEWS - SUPPORTING INDEXES
-- =============================================================================

-- Create indexes to support the views efficiently
CREATE INDEX IF NOT EXISTS idx_blocks_canonical_finalized_chain
    ON blocks(chain_id, canonical, finalized, number DESC);

CREATE INDEX IF NOT EXISTS idx_operator_data_operator_created
    ON operator_data(registration_root, chain_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_operator_collateral_operator
    ON operator_collateral(registration_root, chain_id);
