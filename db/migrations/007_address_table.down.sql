-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON address;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON address;

-- Drop table
DROP TABLE IF EXISTS address CASCADE;
