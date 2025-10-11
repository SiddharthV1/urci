-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON transactions;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON transactions;

-- Drop table
DROP TABLE IF EXISTS transactions CASCADE;
