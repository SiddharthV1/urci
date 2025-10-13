-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON operators;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON operators;

-- Drop table
DROP TABLE IF EXISTS operators CASCADE;
