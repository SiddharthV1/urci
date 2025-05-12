-- Drop the audit log trigger and the updated_at trigger
DROP TRIGGER IF EXISTS audit_log_trigger ON operator_registration;
DROP TRIGGER IF EXISTS set_updated_at ON operator_registration;

DROP TABLE IF EXISTS operator_registrations;

DROP TRIGGER IF EXISTS audit_log_trigger ON operator_registration_events;
DROP TRIGGER IF EXISTS set_updated_at ON operator_registration_events;

DROP TABLE IF EXISTS operator_registration_events;


DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'registration_status'
  ) THEN
    DROP TYPE registration_status;
  END IF;
END $$;
