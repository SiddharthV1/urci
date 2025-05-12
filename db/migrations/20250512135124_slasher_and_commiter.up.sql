DROP TRIGGER IF EXISTS audit_log_trigger ON slasher;
DROP TRIGGER IF EXISTS set_updated_at ON slasher;

DROP TABLE IF EXISTS slasher;

DROP TRIGGER IF EXISTS audit_log_trigger ON commiter;
DROP TRIGGER IF EXISTS set_updated_at ON commiter;

DROP TABLE IF EXISTS commiter;
