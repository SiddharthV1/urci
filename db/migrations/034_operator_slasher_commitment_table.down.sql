-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON operator_slasher_commitment;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON operator_slasher_commitment;

-- Drop table
DROP TABLE IF EXISTS operator_slasher_commitment CASCADE;
