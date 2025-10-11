-- =============================================================================
-- FAILED WRITES FUNCTIONS AND VIEWS (Migration 016 - Part 3)
-- =============================================================================

-- Function to record failed block write
CREATE OR REPLACE FUNCTION record_failed_block_write(
    p_chain_id BIGINT,
    p_block_number BIGINT,
    p_block_hash BYTEA,
    p_block_data JSONB,
    p_events_data JSONB,
    p_error_message TEXT,
    p_error_location TEXT DEFAULT NULL,
    p_error_context JSONB DEFAULT NULL,
    p_writer_id UUID DEFAULT NULL,
    p_session_id TEXT DEFAULT NULL
) RETURNS BIGINT AS $$
DECLARE
    v_id BIGINT;
BEGIN
    INSERT INTO failed_block_writes (
        chain_id, block_number, block_hash,
        block_data, events_data,
        error_message, error_location, error_context,
        writer_id, session_id,
        status, retry_count
    ) VALUES (
        p_chain_id, p_block_number, p_block_hash,
        p_block_data, p_events_data,
        p_error_message, p_error_location, p_error_context,
        p_writer_id, p_session_id,
        'requires_healing', 0
    )
    ON CONFLICT DO NOTHING
    RETURNING id INTO v_id;

    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION record_failed_block_write IS
'Records a failed block write for later healing/retry';

-- View for blocks requiring healing
CREATE OR REPLACE VIEW view_blocks_requiring_healing AS
SELECT
    id,
    chain_id,
    block_number,
    block_hash,
    error_message,
    error_location,
    retry_count,
    max_retries,
    failed_at,
    last_retry_at,
    (max_retries - retry_count) AS retries_remaining
FROM failed_block_writes
WHERE status = 'requires_healing'
  AND retry_count < max_retries
ORDER BY chain_id, block_number;

COMMENT ON VIEW view_blocks_requiring_healing IS
'Shows all blocks that require healing and still have retries available';
