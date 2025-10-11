-- Drop triggers first
DROP TRIGGER IF EXISTS update_beacon_blocks_updated_at ON beacon_blocks;

-- Drop table
DROP TABLE IF EXISTS beacon_blocks CASCADE;
