-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON config;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON config;

-- Drop table
DROP TABLE IF EXISTS config CASCADE;
