-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON slash_offchain_delegation_commitment;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON slash_offchain_delegation_commitment;

-- Drop table
DROP TABLE IF EXISTS slash_offchain_delegation_commitment CASCADE;
