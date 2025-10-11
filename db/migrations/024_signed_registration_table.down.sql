-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON signed_registration;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON signed_registration;

-- Drop table
DROP TABLE IF EXISTS signed_registration CASCADE;
