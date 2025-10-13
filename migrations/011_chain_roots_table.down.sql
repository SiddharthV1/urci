-- Drop triggers first
DROP TRIGGER IF EXISTS set_updated_at_trigger ON chain_roots;

-- Drop table
DROP TABLE IF EXISTS chain_roots CASCADE;
