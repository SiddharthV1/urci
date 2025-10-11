-- =============================================================================
-- WRITER STATUS VIEW
-- =============================================================================

CREATE OR REPLACE FUNCTION stale_update_threshold()
RETURNS INTERVAL AS $$
BEGIN
    RETURN '1 minutes'::interval;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE VIEW view_writer_status AS
SELECT ws.*,
       (CASE
           WHEN ws.seen_block_to IS NULL OR ws.chain_tip IS NULL THEN FALSE
           WHEN ws.seen_block_to >= ws.chain_tip
                AND ws.updated_at >= NOW() - stale_update_threshold() THEN TRUE
           ELSE FALSE
       END) AS synced
FROM writer_status ws;
