-- drop trigger functions
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_proc WHERE proname = 'audit_log_trigger') THEN
        DROP FUNCTION audit_log_trigger() CASCADE;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_proc WHERE proname = 'set_updated_at') THEN
        DROP FUNCTION set_updated_at() CASCADE;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_proc WHERE proname = 'trigger_updated_at') THEN
        DROP FUNCTION trigger_updated_at(regclass) CASCADE;
    END IF;

  --  IF EXISTS (SELECT 1 FROM pg_proc WHERE proname = 'drop_trigger_if_exists') THEN
  --      DROP FUNCTION drop_trigger_if_exists(regclass) CASCADE;
  --  END IF;
END
$$;

-- Drop the audit_log table 
DROP TABLE IF EXISTS audit_log;

-- Drop the writers table 
DROP TABLE IF EXISTS writers;
