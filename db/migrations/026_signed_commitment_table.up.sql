-- =============================================================================
-- SIGNED COMMITMENT TABLE
-- =============================================================================

CREATE TABLE signed_commitment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  schema_id UUID,
  commitment_type INTEGER,
  payload BYTEA,
  slasher_address BYTEA,
  chain_id BIGINT NOT NULL DEFAULT 1,
  commitment JSONB,
  signature UUID,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  event_id BIGINT,
  FOREIGN KEY (slasher_address, chain_id) REFERENCES address(address, chain_id),
  FOREIGN KEY (signature) REFERENCES ecdsa_signature(id),
  FOREIGN KEY (writer_id) REFERENCES writers(id),
  FOREIGN KEY (event_id) REFERENCES events(id)
);

SELECT trigger_audit_log('signed_commitment');
SELECT trigger_updated_at('signed_commitment');
