DROP TRIGGER IF EXISTS audit_log_trigger ON outbox;
DROP TRIGGER IF EXISTS set_updated_at ON outbox;

DROP TABLE IF EXISTS outbox;



DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'outbox_status'
  ) THEN
    DROP TYPE outbox_status;
  END IF;
END $$;

