-- =============================================================================
-- HELPER FUNCTIONS
-- =============================================================================

-- Convert bigint to bytea (big-endian)
CREATE FUNCTION be64(i BIGINT)
RETURNS BYTEA
LANGUAGE SQL IMMUTABLE PARALLEL SAFE STRICT
AS $$ SELECT decode(lpad(to_hex($1), 16, '0'), 'hex') $$;

-- Convert text to UTF8 bytea
CREATE FUNCTION utf8_txt(t TEXT)
RETURNS BYTEA
LANGUAGE SQL IMMUTABLE PARALLEL SAFE STRICT
AS $$ SELECT convert_to($1, 'UTF8') $$;

-- Compute work_id = H(tag || chain_id || height || block_hash || parent_work_id)
CREATE FUNCTION compute_work_id(
    chain_id BIGINT,
    height BIGINT,
    block_hash BYTEA,
    parent_work_id BYTEA
) RETURNS BYTEA
LANGUAGE SQL IMMUTABLE PARALLEL SAFE
AS $$
    SELECT digest(
        utf8_txt('WORK_ID||V1') ||
        be64(chain_id) ||
        be64(height) ||
        block_hash ||
        COALESCE(parent_work_id, '\x0000000000000000000000000000000000000000000000000000000000000000'::bytea),
        'sha256'
    );
$$;

-- Trigger to set updated_at column
CREATE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to attach updated_at trigger to a table
CREATE FUNCTION trigger_updated_at(tablename REGCLASS)
RETURNS VOID AS $$
BEGIN
    EXECUTE format('
        CREATE TRIGGER set_updated_at_trigger
        BEFORE UPDATE ON %s
        FOR EACH ROW
        EXECUTE FUNCTION set_updated_at()',
        tablename
    );
END;
$$ LANGUAGE plpgsql;
