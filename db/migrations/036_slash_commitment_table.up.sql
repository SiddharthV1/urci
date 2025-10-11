-- =============================================================================
-- SLASH COMMITMENT TABLE
-- =============================================================================

CREATE TABLE slash_commitment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  chain_id INTEGER NOT NULL,
  operator_data_id UUID,
  signed_commitment_id UUID,
  committer_id UUID,
  sender_address BYTEA,
  evidence BYTEA,
  sender_reward_wei NUMERIC(78,0),
  burned_wei NUMERIC(78,0),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (operator_data_id) REFERENCES operator_data(id),
  FOREIGN KEY (signed_commitment_id) REFERENCES signed_commitment(id),
  FOREIGN KEY (committer_id) REFERENCES committer(id),
  FOREIGN KEY (sender_address, chain_id) REFERENCES address(address, chain_id),
  FOREIGN KEY (writer_id) REFERENCES writers(id)
);

SELECT trigger_audit_log('slash_commitment');
SELECT trigger_updated_at('slash_commitment');
