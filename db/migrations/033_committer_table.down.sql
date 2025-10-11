-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON committer;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON committer;

-- Drop table
DROP TABLE IF EXISTS committer CASCADE;
