-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON blocks;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON blocks;

-- Drop table
DROP TABLE IF EXISTS blocks CASCADE;
