-- =============================================================================
-- BLOCKS TRIGGER: SET IS_ROOT
-- =============================================================================

-- Trigger to set is_root from chain_roots
CREATE FUNCTION trg_blocks_set_is_root()
RETURNS TRIGGER AS $$
DECLARE
    cfg_start BIGINT;
BEGIN
    SELECT start_height INTO cfg_start FROM chain_roots WHERE chain_id = NEW.chain_id;
    IF cfg_start IS NULL THEN
        RAISE EXCEPTION 'No chain_roots config for chain_id=%', NEW.chain_id;
    END IF;
    NEW.is_root := (NEW.number = cfg_start);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER t_blocks_set_is_root
    BEFORE INSERT OR UPDATE ON blocks
    FOR EACH ROW EXECUTE FUNCTION trg_blocks_set_is_root();
