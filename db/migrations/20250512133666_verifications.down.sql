DROP TRIGGER IF EXISTS audit_log_trigger ON verification;
DROP TRIGGER IF EXISTS set_updated_at ON verification;

DROP TABLE IF EXISTS verification;


DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'verification_status'
  ) THEN
    DROP TYPE verification_status;
  END IF;
END $$;
