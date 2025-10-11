-- =============================================================================
-- WRITER STATUS ENHANCEMENTS (Migration 014 - Part 1)
-- =============================================================================

-- Add unique constraint on session_id
ALTER TABLE writer_status
ADD CONSTRAINT writer_status_session_id_unique UNIQUE (session_id);

-- Add last_indexed_event_id column
ALTER TABLE writer_status
ADD COLUMN last_indexed_event_id BIGINT,
ADD CONSTRAINT fk_writer_status_last_event
    FOREIGN KEY (last_indexed_event_id)
    REFERENCES events(id)
    ON DELETE SET NULL;

CREATE INDEX idx_writer_status_last_event ON writer_status(last_indexed_event_id);

COMMENT ON COLUMN writer_status.last_indexed_event_id IS
'Tracks the most recent event_id indexed by this writer session';
