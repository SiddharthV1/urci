-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON merkle_inclusion_proof;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON merkle_inclusion_proof;

-- Drop table
DROP TABLE IF EXISTS merkle_inclusion_proof CASCADE;
