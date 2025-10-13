-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON operator_collateral;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON operator_collateral;

-- Drop table
DROP TABLE IF EXISTS operator_collateral CASCADE;
