-- =============================================================================
-- EVENTS TABLE
-- =============================================================================

CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    chain_id BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    tx_hash BYTEA NOT NULL,
    tx_index INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    decoded_data JSONB NOT NULL,
    raw_data BYTEA,
    status event_status DEFAULT 'head',
    canonical BOOLEAN DEFAULT TRUE,
    finalized BOOLEAN DEFAULT FALSE,
    writer_id UUID,
    config_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(chain_id, block_hash, tx_index, log_index),
    CONSTRAINT check_block_hash_length CHECK (octet_length(block_hash) = 32),
    CONSTRAINT check_tx_hash_length CHECK (octet_length(tx_hash) = 32),
    FOREIGN KEY (chain_id, block_hash) REFERENCES blocks(chain_id, hash) ON DELETE CASCADE,
    FOREIGN KEY (chain_id, tx_hash) REFERENCES transactions(chain_id, hash) ON DELETE CASCADE,
    FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL,
    FOREIGN KEY (config_id) REFERENCES config(id) ON DELETE SET NULL,
    FOREIGN KEY (chain_id) REFERENCES chain(id)
);

CREATE INDEX idx_events_block ON events(chain_id, block_number, tx_index, log_index);
CREATE INDEX idx_events_tx ON events(chain_id, tx_hash);
CREATE INDEX idx_events_type ON events(chain_id, event_type);
CREATE INDEX idx_events_canonical ON events(chain_id, canonical, finalized);
CREATE INDEX idx_events_status ON events(chain_id, status);

SELECT trigger_audit_log('events');
SELECT trigger_updated_at('events');
