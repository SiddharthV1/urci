-- Drop the audit log trigger and the updated_at trigger
DROP TRIGGER IF EXISTS audit_log_trigger ON config;
DROP TRIGGER IF EXISTS set_updated_at ON config;

-- Drop the table if it exists (includes foreign key to writers)
DROP TABLE IF EXISTS config;


