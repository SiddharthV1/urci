DROP TRIGGER IF EXISTS audit_log_trigger ON keys_and_signatures;
DROP TRIGGER IF EXISTS set_updated_at ON keys_and_signatures;
DROP TABLE IF EXISTS keys_and_signatures;

-- Drop the enum type used by keys_and_signatures.key_type
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'keytype'
  ) THEN
    DROP TYPE keytype;
  END IF;
END $$;
