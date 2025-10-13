-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON signed_delegation;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON signed_delegation;

-- Drop table
DROP TABLE IF EXISTS signed_delegation CASCADE;
