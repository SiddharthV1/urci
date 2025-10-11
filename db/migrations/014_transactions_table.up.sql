-- =============================================================================
-- TRANSACTIONS TABLE
-- =============================================================================

CREATE TABLE transactions (
    hash BYTEA,
    chain_id BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    from_address BYTEA NOT NULL,
    to_address BYTEA,
    value NUMERIC(78,0) DEFAULT 0,
    gas BIGINT NOT NULL,
    gas_price NUMERIC(78,0) NOT NULL,
    nonce BIGINT NOT NULL,
    input BYTEA,
    tx_index INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (chain_id, hash),
    CONSTRAINT check_tx_hash_length CHECK (octet_length(hash) = 32),
    CONSTRAINT check_block_hash_length CHECK (octet_length(block_hash) = 32),
    CONSTRAINT check_from_address_length CHECK (octet_length(from_address) = 20),
    CONSTRAINT check_to_address_length CHECK (to_address IS NULL OR octet_length(to_address) = 20),
    FOREIGN KEY (chain_id, block_hash) REFERENCES blocks(chain_id, hash) ON DELETE CASCADE,
    FOREIGN KEY (from_address, chain_id) REFERENCES address(address, chain_id),
    FOREIGN KEY (to_address, chain_id) REFERENCES address(address, chain_id),
    FOREIGN KEY (chain_id) REFERENCES chain(id)
);

CREATE INDEX idx_transactions_block ON transactions(chain_id, block_hash);
CREATE INDEX idx_transactions_from ON transactions(chain_id, from_address);
CREATE INDEX idx_transactions_to ON transactions(chain_id, to_address) WHERE to_address IS NOT NULL;

SELECT trigger_audit_log('transactions');
SELECT trigger_updated_at('transactions');
