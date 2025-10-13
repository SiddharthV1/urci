-- =============================================================================
-- FAILED BLOCK WRITES TABLE (Migration 016 - Part 1)
-- =============================================================================

CREATE TABLE IF NOT EXISTS failed_block_writes (
    id BIGSERIAL PRIMARY KEY,
    chain_id BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,

    -- Serialized block data for retry
    block_data JSONB NOT NULL,
    events_data JSONB NOT NULL,

    -- Error information
    error_message TEXT NOT NULL,
    error_location TEXT,
    error_context JSONB,

    -- Status tracking
    status TEXT NOT NULL DEFAULT 'requires_healing' CHECK (status IN ('requires_healing', 'healing_in_progress', 'healed', 'permanent_failure')),
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 5,

    -- Metadata
    writer_id UUID,
    session_id TEXT,
    failed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_retry_at TIMESTAMP WITH TIME ZONE,
    healed_at TIMESTAMP WITH TIME ZONE,

    -- Foreign keys
    FOREIGN KEY (chain_id) REFERENCES chain(id),
    FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL
);

CREATE INDEX idx_failed_writes_status ON failed_block_writes(chain_id, status);
CREATE INDEX idx_failed_writes_block ON failed_block_writes(chain_id, block_number);
CREATE INDEX idx_failed_writes_retry ON failed_block_writes(status, retry_count) WHERE status = 'requires_healing';

COMMENT ON TABLE failed_block_writes IS
'Stores blocks that failed to write to allow for healing/retry without crashing the indexer';

COMMENT ON COLUMN failed_block_writes.block_data IS
'Serialized block header data for retry';

COMMENT ON COLUMN failed_block_writes.events_data IS
'Serialized events from the block for retry';
