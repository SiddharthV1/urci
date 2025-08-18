-- Database improvements migration
-- This implements all improvements from DATABASE_IMPROVEMENTS.md

-- 1. Add missing kafka_offsets table
CREATE TABLE IF NOT EXISTS kafka_offsets (
    cg TEXT NOT NULL,
    topic TEXT NOT NULL,
    partition INT NOT NULL,
    offset BIGINT NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (cg, topic, partition)
);

-- 2. Add performance indexes
CREATE INDEX IF NOT EXISTS idx_events_block_number ON events(block_number);
CREATE INDEX IF NOT EXISTS idx_events_tx_hash ON events(tx_hash);
CREATE INDEX IF NOT EXISTS idx_events_status ON events(status);
CREATE INDEX IF NOT EXISTS idx_events_block_status ON events(block_number, status);
CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_operators_registration_root ON operators(registration_root);
CREATE INDEX IF NOT EXISTS idx_operators_address ON operators(address);
CREATE INDEX IF NOT EXISTS idx_operators_created_at ON operators(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_operator_collateral_operator_id ON operator_collateral(operator_id);
CREATE INDEX IF NOT EXISTS idx_operator_collateral_created_at ON operator_collateral(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_operator_registrations_event_operator ON operator_registrations(event_id, operator_id);
CREATE INDEX IF NOT EXISTS idx_operator_registration_events_operator ON operator_registration_events(operator_id);
CREATE INDEX IF NOT EXISTS idx_operator_registration_events_status ON operator_registration_events(status);

CREATE INDEX IF NOT EXISTS idx_slashing_events_operator_id ON slashing_events(operator_id);
CREATE INDEX IF NOT EXISTS idx_slashing_events_type ON slashing_events(slashing_type);
CREATE INDEX IF NOT EXISTS idx_slashing_events_created_at ON slashing_events(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_slasher_commitments_operator_id ON slasher_commitments(operator_id);
CREATE INDEX IF NOT EXISTS idx_slasher_commitments_slasher_id ON slasher_commitments(slasher_id);

CREATE INDEX IF NOT EXISTS idx_slasher_registration_events_operator_id ON slasher_registration_events(operator_id);
CREATE INDEX IF NOT EXISTS idx_slasher_registration_events_slasher ON slasher_registration_events(slasher_address);

CREATE INDEX IF NOT EXISTS idx_collateral_transfer_events_operator_id ON collateral_transfer_events(operator_id);
CREATE INDEX IF NOT EXISTS idx_collateral_transfer_events_type ON collateral_transfer_events(transfer_type);

CREATE INDEX IF NOT EXISTS idx_outbox_status ON outbox(status);
CREATE INDEX IF NOT EXISTS idx_outbox_created_at ON outbox(created_at);

-- 3. Add missing relationships and constraints
ALTER TABLE slasher_commitments 
    ADD COLUMN IF NOT EXISTS slasher_table_id UUID REFERENCES slasher(id),
    ADD COLUMN IF NOT EXISTS committer_table_id UUID REFERENCES committer(id);

-- Add constraints for data integrity
ALTER TABLE operator_collateral 
    ADD CONSTRAINT positive_collateral CHECK (collateral_wei >= 0);

ALTER TABLE events 
    ADD CONSTRAINT valid_event_status CHECK (status IN ('not_finalized', 'finalized', 'rolledback'));

-- Add unique constraints to prevent duplicates
ALTER TABLE operators 
    ADD CONSTRAINT unique_registration_root UNIQUE (registration_root);

ALTER TABLE slasher_commitments
    ADD CONSTRAINT unique_operator_slasher UNIQUE (operator_id, slasher_id);

-- 4. Add missing tables for complete event support

-- Delegation tracking
CREATE TABLE IF NOT EXISTS delegations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id UUID REFERENCES operators(id),
    slot BIGINT NOT NULL,
    delegate_address CHAR(42),
    committer_address CHAR(42),
    delegation_data JSONB,
    event_id UUID REFERENCES events(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    writer_id UUID REFERENCES writers(id)
);

-- Equivocation tracking
CREATE TABLE IF NOT EXISTS equivocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id UUID REFERENCES operators(id),
    evidence JSONB NOT NULL,
    first_delegation JSONB,
    second_delegation JSONB,
    event_id UUID REFERENCES events(id),
    created_at TIMESTAMP DEFAULT NOW(),
    writer_id UUID REFERENCES writers(id)
);

-- Registration proof tracking
CREATE TABLE IF NOT EXISTS registration_proofs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id UUID REFERENCES operators(id),
    proof_data JSONB NOT NULL,
    merkle_path JSONB,
    verified BOOLEAN DEFAULT false,
    event_id UUID REFERENCES events(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    writer_id UUID REFERENCES writers(id)
);

-- Operator deletion tracking
CREATE TABLE IF NOT EXISTS operator_deletions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id UUID REFERENCES operators(id),
    reason TEXT,
    deleted_by TEXT,
    event_id UUID REFERENCES events(id),
    created_at TIMESTAMP DEFAULT NOW(),
    writer_id UUID REFERENCES writers(id)
);

-- Add indexes for new tables
CREATE INDEX idx_delegations_operator ON delegations(operator_id);
CREATE INDEX idx_delegations_slot ON delegations(slot);
CREATE INDEX idx_equivocations_operator ON equivocations(operator_id);
CREATE INDEX idx_registration_proofs_operator ON registration_proofs(operator_id);
CREATE INDEX idx_registration_proofs_verified ON registration_proofs(verified);
CREATE INDEX idx_operator_deletions_operator ON operator_deletions(operator_id);

-- 5. Create materialized views for performance

-- Current operator state view
CREATE MATERIALIZED VIEW IF NOT EXISTS operator_current_state AS
SELECT DISTINCT ON (o.id)
    o.id,
    o.registration_root,
    o.address,
    oc.collateral_wei as current_collateral,
    ore.status as registration_status,
    COUNT(DISTINCT se.id) as slash_count,
    COUNT(DISTINCT sc.id) as active_slashers,
    MAX(se.created_at) as last_slashed_at,
    o.created_at as registered_at,
    CASE 
        WHEN ore.status = 'unregister' THEN ore.created_at
        ELSE NULL
    END as unregistered_at
FROM operators o
LEFT JOIN LATERAL (
    SELECT collateral_wei 
    FROM operator_collateral 
    WHERE operator_id = o.id 
    ORDER BY created_at DESC 
    LIMIT 1
) oc ON true
LEFT JOIN LATERAL (
    SELECT status, created_at
    FROM operator_registration_events
    WHERE operator_id = o.id
    ORDER BY created_at DESC
    LIMIT 1
) ore ON true
LEFT JOIN slashing_events se ON o.id = se.operator_id
LEFT JOIN slasher_commitments sc ON o.id = sc.operator_id AND sc.opted_in_at > COALESCE(sc.opted_out_at, '1970-01-01')
GROUP BY o.id, o.registration_root, o.address, oc.collateral_wei, ore.status, ore.created_at
ORDER BY o.id, o.created_at DESC;

-- Create index on materialized view
CREATE UNIQUE INDEX idx_operator_current_state_id ON operator_current_state(id);
CREATE INDEX idx_operator_current_state_registration_root ON operator_current_state(registration_root);
CREATE INDEX idx_operator_current_state_status ON operator_current_state(registration_status);

-- Slashing statistics view
CREATE MATERIALIZED VIEW IF NOT EXISTS slashing_statistics AS
SELECT 
    DATE_TRUNC('day', se.created_at) as day,
    se.slashing_type,
    COUNT(*) as slash_count,
    SUM(se.amount) as total_amount,
    AVG(se.amount) as avg_amount,
    MAX(se.amount) as max_amount,
    MIN(se.amount) as min_amount
FROM slashing_events se
GROUP BY DATE_TRUNC('day', se.created_at), se.slashing_type
ORDER BY day DESC, se.slashing_type;

CREATE INDEX idx_slashing_statistics_day ON slashing_statistics(day);
CREATE INDEX idx_slashing_statistics_type ON slashing_statistics(slashing_type);

-- Active slashers view
CREATE MATERIALIZED VIEW IF NOT EXISTS active_slashers AS
SELECT 
    s.address as slasher_address,
    COUNT(DISTINCT sc.operator_id) as operator_count,
    COUNT(DISTINCT se.id) as slash_count,
    SUM(se.amount) as total_slashed,
    MAX(se.created_at) as last_slash_at
FROM slasher s
JOIN slasher_commitments sc ON s.id = sc.slasher_table_id
LEFT JOIN slashing_events se ON sc.operator_id = se.operator_id
WHERE sc.opted_in_at > COALESCE(sc.opted_out_at, '1970-01-01')
GROUP BY s.address;

CREATE INDEX idx_active_slashers_address ON active_slashers(slasher_address);
CREATE INDEX idx_active_slashers_operator_count ON active_slashers(operator_count DESC);

-- 6. Functions for materialized view refresh

CREATE OR REPLACE FUNCTION refresh_materialized_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY operator_current_state;
    REFRESH MATERIALIZED VIEW CONCURRENTLY slashing_statistics;
    REFRESH MATERIALIZED VIEW CONCURRENTLY active_slashers;
END;
$$ LANGUAGE plpgsql;

-- 7. Partitioning for large tables (events by month)

-- Create partitioned events table
CREATE TABLE IF NOT EXISTS events_partitioned (
    LIKE events INCLUDING ALL
) PARTITION BY RANGE (created_at);

-- Create partitions for the next 12 months
DO $$
DECLARE
    start_date DATE := DATE_TRUNC('month', CURRENT_DATE);
    end_date DATE;
    partition_name TEXT;
BEGIN
    FOR i IN 0..11 LOOP
        end_date := start_date + INTERVAL '1 month';
        partition_name := 'events_' || TO_CHAR(start_date, 'YYYY_MM');
        
        EXECUTE format('
            CREATE TABLE IF NOT EXISTS %I PARTITION OF events_partitioned
            FOR VALUES FROM (%L) TO (%L)',
            partition_name, start_date, end_date
        );
        
        start_date := end_date;
    END LOOP;
END $$;

-- 8. Add triggers for new tables
SELECT trigger_updated_at('delegations');
SELECT trigger_updated_at('registration_proofs');
SELECT trigger_audit_log('delegations');
SELECT trigger_audit_log('equivocations');
SELECT trigger_audit_log('registration_proofs');
SELECT trigger_audit_log('operator_deletions');

-- 9. Add foreign key cascades for better data integrity
ALTER TABLE operator_collateral
    DROP CONSTRAINT IF EXISTS operator_collateral_operator_id_fkey,
    ADD CONSTRAINT operator_collateral_operator_id_fkey 
        FOREIGN KEY (operator_id) REFERENCES operators(id) ON DELETE CASCADE;

ALTER TABLE operator_registration_events
    DROP CONSTRAINT IF EXISTS operator_registration_events_operator_id_fkey,
    ADD CONSTRAINT operator_registration_events_operator_id_fkey
        FOREIGN KEY (operator_id) REFERENCES operators(id) ON DELETE CASCADE;

ALTER TABLE slashing_events
    DROP CONSTRAINT IF EXISTS slashing_events_operator_id_fkey,
    ADD CONSTRAINT slashing_events_operator_id_fkey
        FOREIGN KEY (operator_id) REFERENCES operators(id) ON DELETE CASCADE;

-- 10. Create function to get operator history
CREATE OR REPLACE FUNCTION get_operator_history(p_registration_root VARCHAR)
RETURNS TABLE (
    event_type TEXT,
    event_time TIMESTAMP,
    collateral_change BIGINT,
    current_collateral BIGINT,
    details JSONB
) AS $$
BEGIN
    RETURN QUERY
    WITH operator_info AS (
        SELECT id FROM operators WHERE registration_root = p_registration_root
    ),
    events_timeline AS (
        -- Registration events
        SELECT 
            'registration' as event_type,
            ore.created_at as event_time,
            ore.collateral as collateral_change,
            ore.collateral as current_collateral,
            jsonb_build_object('status', ore.status) as details
        FROM operator_registration_events ore
        JOIN operator_info oi ON ore.operator_id = oi.id
        
        UNION ALL
        
        -- Collateral changes
        SELECT 
            'collateral_change' as event_type,
            oc.created_at as event_time,
            oc.collateral_wei - LAG(oc.collateral_wei, 1, 0) OVER (ORDER BY oc.created_at) as collateral_change,
            oc.collateral_wei as current_collateral,
            jsonb_build_object('event_id', oc.event_id) as details
        FROM operator_collateral oc
        JOIN operator_info oi ON oc.operator_id = oi.id
        
        UNION ALL
        
        -- Slashing events
        SELECT 
            'slashing' as event_type,
            se.created_at as event_time,
            -se.amount as collateral_change,
            NULL as current_collateral,
            jsonb_build_object(
                'type', se.slashing_type,
                'amount', se.amount,
                'challenger', se.challenger
            ) as details
        FROM slashing_events se
        JOIN operator_info oi ON se.operator_id = oi.id
    )
    SELECT * FROM events_timeline
    ORDER BY event_time ASC;
END;
$$ LANGUAGE plpgsql;

-- 11. Create scheduled job for cleanup and maintenance
CREATE OR REPLACE FUNCTION maintenance_job()
RETURNS void AS $$
BEGIN
    -- Refresh materialized views
    PERFORM refresh_materialized_views();
    
    -- Cleanup old notification data
    DELETE FROM notification_deliveries
    WHERE created_at < NOW() - INTERVAL '30 days'
      AND delivery_status IN ('sent', 'failed');
    
    -- Cleanup old rate limits
    DELETE FROM notification_rate_limits
    WHERE window_start < NOW() - INTERVAL '1 day';
    
    -- Vacuum analyze for performance
    VACUUM ANALYZE;
END;
$$ LANGUAGE plpgsql;

-- Create extension for scheduling if not exists
CREATE EXTENSION IF NOT EXISTS pg_cron;

-- Schedule maintenance job to run daily at 3 AM
-- SELECT cron.schedule('maintenance-job', '0 3 * * *', 'SELECT maintenance_job();');