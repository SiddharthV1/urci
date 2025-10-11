-- =============================================================================
-- BEACON BLOCKS TABLE
-- Maps beacon chain data to execution layer blocks for correlation
-- =============================================================================

CREATE TABLE IF NOT EXISTS beacon_blocks (
    -- Beacon chain identifiers
    slot BIGINT NOT NULL,
    epoch BIGINT NOT NULL,
    beacon_root VARCHAR(66) NOT NULL,
    beacon_state_root VARCHAR(66) NOT NULL,

    -- Chain reference
    chain_id BIGINT NOT NULL,

    -- Execution layer correlation
    execution_block_hash VARCHAR(66) NOT NULL,
    execution_block_number BIGINT NOT NULL,

    -- Finalization tracking
    is_finalized BOOLEAN NOT NULL DEFAULT FALSE,
    is_justified BOOLEAN NOT NULL DEFAULT FALSE,
    is_head BOOLEAN NOT NULL DEFAULT FALSE,

    -- Canonical tracking
    canonical BOOLEAN NOT NULL DEFAULT TRUE,

    -- Timestamps
    beacon_timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    PRIMARY KEY (slot, beacon_root),
    FOREIGN KEY (chain_id) REFERENCES chain(id)
);

-- Indexes for efficient lookups
CREATE INDEX idx_beacon_blocks_execution_hash ON beacon_blocks(execution_block_hash);
CREATE INDEX idx_beacon_blocks_execution_number ON beacon_blocks(execution_block_number);
CREATE INDEX idx_beacon_blocks_epoch ON beacon_blocks(epoch);
CREATE INDEX idx_beacon_blocks_finalized ON beacon_blocks(is_finalized) WHERE is_finalized = TRUE;
CREATE INDEX idx_beacon_blocks_canonical ON beacon_blocks(canonical) WHERE canonical = TRUE;
CREATE INDEX idx_beacon_blocks_head ON beacon_blocks(is_head) WHERE is_head = TRUE;

-- Trigger to update timestamps
CREATE TRIGGER update_beacon_blocks_updated_at
    BEFORE UPDATE ON beacon_blocks
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
