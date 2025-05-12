DROP TRIGGER IF EXISTS audit_log_trigger ON slasher;
DROP TRIGGER IF EXISTS set_updated_at ON slasher;

DROP TABLE IF EXISTS slasher;

DROP TRIGGER IF EXISTS audit_log_trigger ON committer;
DROP TRIGGER IF EXISTS set_updated_at ON committer;

DROP TABLE IF EXISTS committer;



