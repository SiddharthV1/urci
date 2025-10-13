-- =============================================================================
-- MERKLE TREE TABLE
-- =============================================================================

CREATE TABLE merkle_tree (
    root BYTEA PRIMARY KEY,
    leafs JSONB NOT NULL,
    leaf_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_root_length CHECK (octet_length(root) = 32)
);

SELECT trigger_audit_log('merkle_tree');
SELECT trigger_updated_at('merkle_tree');
