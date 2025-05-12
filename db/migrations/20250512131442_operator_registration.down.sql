-- Drop the audit log trigger and the updated_at trigger
DROP TRIGGER IF EXISTS audit_log_trigger ON operator_registration;
DROP TRIGGER IF EXISTS set_updated_at ON operator_registration;

DROP TABLE IF EXISTS operator_registrations;



