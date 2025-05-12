DROP TRIGGER IF EXISTS audit_log_trigger ON slasher_commitment;
DROP TRIGGER IF EXISTS set_updated_at ON slasher_commitment;

DROP TABLE IF EXISTS slasher_commitment;

DROP TRIGGER IF EXISTS audit_log_trigger ON operator_slasher_commitments;
DROP TRIGGER IF EXISTS set_updated_at ON operator_slasher_commitments;

DROP TABLE IF EXISTS operator_slasher_commitments;
