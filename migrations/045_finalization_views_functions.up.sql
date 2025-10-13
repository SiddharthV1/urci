-- =============================================================================
-- FINALIZATION VIEWS AND FUNCTIONS (Migration 014 - Part 3)
-- =============================================================================

-- View for last finalized block by chain
CREATE OR REPLACE VIEW view_last_finalized_block_by_chain AS
SELECT
    chain_id,
    number,
    hash,
    work_id,
    timestamp
FROM blocks b
WHERE finalized = true
  AND (chain_id, number) IN (
      SELECT chain_id, MAX(number)
      FROM blocks
      WHERE finalized = true
      GROUP BY chain_id
  );

COMMENT ON VIEW view_last_finalized_block_by_chain IS
'Returns the last (highest) finalized block for each chain_id';

-- Function to finalize and clean up empty blocks
CREATE OR REPLACE FUNCTION finalize_session_blocks(
    p_chain_id BIGINT,
    p_finalize_up_to_block BIGINT
) RETURNS void AS $$
DECLARE
    empty_block_to_delete RECORD;
BEGIN
    -- Step 1: Mark blocks WITH events as finalized
    UPDATE blocks
    SET finalized = true
    WHERE chain_id = p_chain_id
      AND number <= p_finalize_up_to_block
      AND canonical = true
      AND hash IN (
          SELECT DISTINCT block_hash
          FROM events
          WHERE chain_id = p_chain_id
      );

    -- Step 2: Recompute parent_work_id for blocks that reference empty blocks
    UPDATE blocks b
    SET
        parent_work_id = (
            SELECT work_id
            FROM blocks prev
            WHERE prev.chain_id = p_chain_id
              AND prev.number < b.number
              AND prev.finalized = true
            ORDER BY prev.number DESC
            LIMIT 1
        ),
        is_root = CASE
            WHEN NOT EXISTS (
                SELECT 1
                FROM blocks prev
                WHERE prev.chain_id = p_chain_id
                  AND prev.number < b.number
                  AND prev.finalized = true
            ) THEN true
            ELSE false
        END
    WHERE b.chain_id = p_chain_id
      AND b.parent_work_id IN (
          SELECT work_id
          FROM blocks empty
          WHERE empty.chain_id = p_chain_id
            AND empty.number <= p_finalize_up_to_block
            AND empty.canonical = true
            AND empty.hash NOT IN (
                SELECT DISTINCT block_hash
                FROM events
                WHERE chain_id = p_chain_id
            )
      );

    -- Step 3: Delete empty blocks in reverse order
    FOR empty_block_to_delete IN
        SELECT work_id
        FROM blocks
        WHERE chain_id = p_chain_id
          AND number <= p_finalize_up_to_block
          AND canonical = true
          AND hash NOT IN (
              SELECT DISTINCT block_hash
              FROM events
              WHERE chain_id = p_chain_id
          )
        ORDER BY number DESC
    LOOP
        DELETE FROM blocks WHERE work_id = empty_block_to_delete.work_id;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION finalize_session_blocks IS
'Finalizes blocks up to a given block number for a chain: marks blocks with events as finalized and deletes empty blocks';

-- View for fork detection using active_event_id
CREATE OR REPLACE VIEW view_fork_head_events AS
WITH synced_writers AS (
    SELECT chain_id, MIN(seen_block_to) as highest_common_block
    FROM view_writer_status
    WHERE synced = true
    GROUP BY chain_id
)
SELECT
    b.chain_id,
    b.number,
    b.hash,
    b.active_event_id,
    b.canonical,
    b.finalized,
    sw.highest_common_block
FROM blocks b
INNER JOIN synced_writers sw ON b.chain_id = sw.chain_id AND b.number = sw.highest_common_block
WHERE b.active_event_id IS NOT NULL
  AND b.canonical = true
ORDER BY b.chain_id, b.active_event_id;

COMMENT ON VIEW view_fork_head_events IS
'Shows fork head events at the highest common synced block across all synced writers. Multiple rows for the same chain indicate a fork.';
