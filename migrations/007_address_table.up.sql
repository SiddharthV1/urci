-- =============================================================================
-- ADDRESS TABLE
-- =============================================================================

CREATE TABLE address (
    address BYTEA,
    chain_id BIGINT NOT NULL,
    is_contract BOOLEAN DEFAULT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_address_length CHECK (octet_length(address) = 20),
    CONSTRAINT pk_address PRIMARY KEY (address, chain_id),
    FOREIGN KEY (chain_id) REFERENCES chain(id)
);

CREATE INDEX idx_address_is_contract ON address(is_contract);

SELECT trigger_audit_log('address');
SELECT trigger_updated_at('address');
