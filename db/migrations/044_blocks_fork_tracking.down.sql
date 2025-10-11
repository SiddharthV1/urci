DROP TRIGGER IF EXISTS blocks_session_id_check ON blocks;
DROP FUNCTION IF EXISTS check_blocks_session_id();

ALTER TABLE blocks
DROP CONSTRAINT IF EXISTS fk_blocks_active_event,
DROP COLUMN IF EXISTS active_event_id,
DROP COLUMN IF EXISTS session_id;
