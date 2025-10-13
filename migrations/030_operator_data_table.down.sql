-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON operator_data;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON operator_data;

-- Drop table
DROP TABLE IF EXISTS operator_data CASCADE;
