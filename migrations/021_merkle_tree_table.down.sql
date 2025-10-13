-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON merkle_tree;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON merkle_tree;

-- Drop table
DROP TABLE IF EXISTS merkle_tree CASCADE;
