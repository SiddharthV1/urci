DROP TRIGGER IF EXISTS audit_log_trigger ON collateral_transfer_events;
DROP TRIGGER IF EXISTS set_updated_at ON collateral_transfer_events;

DROP TABLE IF EXISTS collateral_transfer_events;

DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'transfer_type'
  ) THEN
    DROP TYPE transfer_type;
  END IF;
END $$;
