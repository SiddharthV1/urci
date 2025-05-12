DROP TRIGGER IF EXISTS audit_log_trigger ON slasher_registration_events;
DROP TRIGGER IF EXISTS set_updated_at ON slasher_registration_events;

DROP TABLE IF EXISTS slasher_registration_events;



DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'slasher_commitment_status'
  ) THEN
    DROP TYPE slasher_commitment_status;
  END IF;
END $$;

