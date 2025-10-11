-- =============================================================================
-- BLOCKS FORK TRACKING (Migration 014 - Part 2)
-- =============================================================================

-- Add active_event_id and session_id to blocks table
ALTER TABLE blocks
ADD COLUMN active_event_id BIGINT,
ADD COLUMN session_id TEXT;

-- Add FK constraint for active_event_id
ALTER TABLE blocks
ADD CONSTRAINT fk_blocks_active_event
    FOREIGN KEY (active_event_id)
    REFERENCES events(id)
    ON DELETE SET NULL;

-- Create trigger function to validate session_id
CREATE OR REPLACE FUNCTION check_blocks_session_id()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.session_id IS NOT NULL THEN
        IF NOT EXISTS (SELECT 1 FROM writer_status WHERE session_id = NEW.session_id) THEN
            RAISE EXCEPTION 'session_id % does not exist in writer_status', NEW.session_id;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER blocks_session_id_check
BEFORE INSERT OR UPDATE ON blocks
FOR EACH ROW
EXECUTE FUNCTION check_blocks_session_id();

CREATE INDEX idx_blocks_active_event ON blocks(chain_id, number, active_event_id);
CREATE INDEX idx_blocks_session ON blocks(session_id);

COMMENT ON COLUMN blocks.active_event_id IS
'Tracks the active/head event for this block to detect forks when multiple writers see different events at the same block height';

COMMENT ON COLUMN blocks.session_id IS
'Tracks which writer session wrote this block for multi-writer coordination and reorg handling';
