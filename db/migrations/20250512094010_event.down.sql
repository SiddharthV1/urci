-- Drop the audit log trigger and the updated_at trigger
DROP TRIGGER IF EXISTS audit_log_trigger ON events;
DROP TRIGGER IF EXISTS set_updated_at ON events;

-- Drop the table if it exists (includes foreign key to writers)
DROP TABLE IF EXISTS events;

-- Drop the enum type used by events.status
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'event_status'
  ) THEN
    DROP TYPE event_status;
  END IF;
END $$;


