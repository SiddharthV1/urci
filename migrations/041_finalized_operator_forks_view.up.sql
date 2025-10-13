-- =============================================================================
-- FINALIZED OPERATOR FORKS VIEW
-- Shows operators at finalized chain tips only
-- =============================================================================

CREATE VIEW finalized_operator_forks AS
WITH finalized_blocks AS (
    -- Get the latest unique work_ids for finalized blocks only
    SELECT DISTINCT ON (chain_id)
        chain_id,
        work_id,
        number as block_number,
        hash as block_hash,
        timestamp as block_timestamp,
        canonical,
        finalized,
        parent_work_id
    FROM blocks
    WHERE canonical = TRUE
      AND finalized = TRUE
    ORDER BY chain_id, number DESC
),
writer_sync_status AS (
    -- Get most recent sync status from each writer for each chain
    WITH latest_writer_status AS (
        SELECT DISTINCT ON (vws.writer_id, vws.chain_id)
            vws.writer_id,
            vws.chain_id,
            vws.chain_tip,
            vws.seen_block_to,
            vws.synced,
            vws.updated_at
        FROM view_writer_status vws
        ORDER BY vws.writer_id, vws.chain_id, vws.updated_at DESC
    ),
    -- Get the absolute latest status per chain
    most_recent_per_chain AS (
        SELECT DISTINCT ON (chain_id)
            chain_id,
            chain_tip as latest_chain_tip,
            seen_block_to as latest_seen_block
        FROM latest_writer_status
        ORDER BY chain_id, updated_at DESC
    )
    SELECT
        lws.chain_id,
        mrc.latest_chain_tip as max_chain_tip,
        mrc.latest_seen_block as max_seen_block,
        BOOL_OR(lws.synced) as any_writer_synced,
        BOOL_AND(lws.synced) as all_writers_synced,
        COUNT(DISTINCT lws.writer_id) as writer_count,
        COUNT(DISTINCT lws.writer_id) FILTER (WHERE lws.synced = TRUE) as synced_writer_count,
        MAX(lws.updated_at) as last_update
    FROM latest_writer_status lws
    JOIN most_recent_per_chain mrc ON lws.chain_id = mrc.chain_id
    GROUP BY lws.chain_id, mrc.latest_chain_tip, mrc.latest_seen_block
),
operator_states AS (
    -- Get operator states at finalized blocks only
    SELECT DISTINCT ON (o.registration_root, o.chain_id, fb.work_id)
        o.registration_root,
        o.chain_id,
        o.owner_address,
        o.num_keys,
        o.registration_processed,
        fb.work_id,
        fb.block_number,
        fb.block_hash,
        fb.block_timestamp,
        oc.collateral_wei_total,
        ore.event_type as last_event_type,
        ore.registered_at,
        ore.unregistered_at,
        ore.slashed_at,
        ore.deleted,
        ore.equivocated
    FROM operators o
    LEFT JOIN operator_collateral oc ON oc.registration_root = o.registration_root AND oc.chain_id = o.chain_id
    LEFT JOIN operator_data ore ON ore.registration_root = o.registration_root AND ore.chain_id = o.chain_id
    CROSS JOIN finalized_blocks fb
    WHERE o.created_at IS NOT NULL
    ORDER BY o.registration_root, o.chain_id, fb.work_id, ore.created_at DESC
)
SELECT
    os.*,
    fb.canonical,
    fb.finalized,
    fb.parent_work_id,
    -- Sync status from writers (finalized blocks should always be synced)
    TRUE as is_synced,
    wss.all_writers_synced,
    wss.writer_count,
    wss.synced_writer_count,
    wss.max_chain_tip,
    wss.max_seen_block,
    wss.last_update as sync_last_update,
    -- Fork detection (finalized blocks are stable)
    'finalized' as sync_status,
    -- Chain tip distance for finalized blocks
    CASE
        WHEN wss.max_chain_tip IS NOT NULL AND os.block_number IS NOT NULL
        THEN wss.max_chain_tip - os.block_number
        ELSE NULL
    END as blocks_behind_tip
FROM operator_states os
JOIN finalized_blocks fb ON os.work_id = fb.work_id
LEFT JOIN writer_sync_status wss ON os.chain_id = wss.chain_id
ORDER BY os.chain_id, os.registration_root;
