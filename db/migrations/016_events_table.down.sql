-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON events;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON events;

-- Drop table
DROP TABLE IF EXISTS events CASCADE;
