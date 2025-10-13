-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON slash_registration;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON slash_registration;

-- Drop table
DROP TABLE IF EXISTS slash_registration CASCADE;
