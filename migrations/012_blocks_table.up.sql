-- =============================================================================
-- BLOCKS TABLE
-- =============================================================================

CREATE TABLE blocks (
    hash BYTEA,
    chain_id BIGINT NOT NULL,
    number BIGINT NOT NULL,
    parent_hash BYTEA NOT NULL,
    parent_work_id BYTEA,
    timestamp BIGINT NOT NULL,
    canonical BOOLEAN DEFAULT TRUE,
    finalized BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    work_id BYTEA GENERATED ALWAYS AS (
        compute_work_id(chain_id, number, hash, parent_work_id)
    ) STORED,
    is_root BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (hash, chain_id),
    CONSTRAINT uidx_block_work_id UNIQUE (work_id),
    CONSTRAINT check_hash_length CHECK (octet_length(hash) = 32),
    CONSTRAINT check_parent_hash_length CHECK (octet_length(parent_hash) = 32),
    CONSTRAINT chk_blocks_parent_null_only_on_root CHECK (
        NOT is_root OR parent_work_id IS NULL
    ),
    FOREIGN KEY (parent_work_id) REFERENCES blocks(work_id) DEFERRABLE INITIALLY DEFERRED
);

CREATE INDEX idx_blocks_hash ON blocks USING HASH (hash);
CREATE INDEX idx_blocks_number ON blocks(chain_id, number DESC);
CREATE INDEX idx_blocks_canonical ON blocks(canonical, finalized, number DESC);
CREATE UNIQUE INDEX uidx_blocks_one_root_per_chain ON blocks(chain_id) WHERE is_root;

SELECT trigger_audit_log('blocks');
SELECT trigger_updated_at('blocks');
