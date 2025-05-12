DROP TRIGGER IF EXISTS audit_log_trigger ON slashing_events;
DROP TRIGGER IF EXISTS set_updated_at ON slashing_events;

DROP TABLE IF EXISTS slashing_events;



DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'slashing_type'
  ) THEN
    DROP TYPE slashing_type;
  END IF;
END $$;

