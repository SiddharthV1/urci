-- =============================================================================
-- CONFIGURATION TABLE
-- =============================================================================

-- Registry configuration
CREATE TABLE config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chain_id BIGINT NOT NULL,
    contract_address BYTEA NOT NULL,
    min_collateral_wei NUMERIC(78,0) NOT NULL,
    fraud_proof_window INTEGER NOT NULL,
    unregistration_delay INTEGER NOT NULL,
    slash_window INTEGER NOT NULL,
    opt_in_delay INTEGER NOT NULL,
    writer_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_address_length CHECK (octet_length(contract_address) = 20),
    CONSTRAINT check_positive_values CHECK (
        min_collateral_wei >= 0 AND
        fraud_proof_window > 0 AND
        unregistration_delay > 0 AND
        slash_window > 0 AND
        opt_in_delay >= 0
    ),
    CONSTRAINT unique_config_per_chain UNIQUE (chain_id, contract_address),
    FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL,
    FOREIGN KEY (contract_address, chain_id) REFERENCES address(address, chain_id),
    FOREIGN KEY (chain_id) REFERENCES chain(id)
);

CREATE INDEX idx_config_contract_address ON config(contract_address);

SELECT trigger_audit_log('config');
SELECT trigger_updated_at('config');
