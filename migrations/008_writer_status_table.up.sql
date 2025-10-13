-- =============================================================================
-- WRITER STATUS TABLE
-- =============================================================================

CREATE TABLE writer_status (
    writer_id UUID NOT NULL,
    session_id TEXT NOT NULL,
    chain_id BIGINT NOT NULL,
    seen_block_from BIGINT,
    seen_block_to BIGINT,
    chain_tip BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (writer_id, session_id, chain_id),
    FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE CASCADE
);

CREATE INDEX idx_writer_status_writer_id ON writer_status(writer_id);
