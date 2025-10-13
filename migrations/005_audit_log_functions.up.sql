-- =============================================================================
-- AUDIT LOG FUNCTIONS
-- =============================================================================

-- Audit log trigger function
CREATE FUNCTION audit_log_trigger()
RETURNS TRIGGER AS $$
DECLARE
    v_action VARCHAR(10);
    v_record_id TEXT;
    v_old_data JSONB;
    v_new_data JSONB;
BEGIN
    v_action := TG_OP;

    IF TG_OP = 'DELETE' THEN
        v_old_data := to_jsonb(OLD);
        v_new_data := NULL;
        v_record_id := v_old_data->>'id';
        IF v_record_id IS NULL THEN
            v_record_id := md5(v_old_data::TEXT);
        END IF;
    ELSIF TG_OP = 'INSERT' THEN
        v_old_data := NULL;
        v_new_data := to_jsonb(NEW);
        v_record_id := v_new_data->>'id';
        IF v_record_id IS NULL THEN
            v_record_id := md5(v_new_data::TEXT);
        END IF;
    ELSE -- UPDATE
        v_old_data := to_jsonb(OLD);
        v_new_data := to_jsonb(NEW);
        v_record_id := v_new_data->>'id';
        IF v_record_id IS NULL THEN
            v_record_id := md5(v_new_data::TEXT);
        END IF;
    END IF;

    INSERT INTO audit_log (action, table_name, record_id, old_data, new_data, writer_id)
    VALUES (v_action, TG_TABLE_NAME, v_record_id, v_old_data, v_new_data, (v_new_data->>'writer_id')::UUID);

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to attach audit log trigger to a table
CREATE FUNCTION trigger_audit_log(tablename REGCLASS)
RETURNS VOID AS $$
BEGIN
    EXECUTE format('
        CREATE TRIGGER audit_log_trigger
        AFTER INSERT OR UPDATE OR DELETE ON %s
        FOR EACH ROW
        EXECUTE FUNCTION audit_log_trigger()',
        tablename
    );
END;
$$ LANGUAGE plpgsql;
