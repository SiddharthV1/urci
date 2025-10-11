ALTER TABLE writer_status
DROP CONSTRAINT IF EXISTS fk_writer_status_last_event,
DROP COLUMN IF EXISTS last_indexed_event_id,
DROP CONSTRAINT IF EXISTS writer_status_session_id_unique;
