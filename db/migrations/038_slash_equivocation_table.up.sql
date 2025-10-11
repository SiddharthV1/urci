-- =============================================================================
-- SLASH EQUIVOCATION TABLE
-- =============================================================================

CREATE TABLE slash_equivocation (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  chain_id INTEGER NOT NULL,
  operator_data_id UUID,
  signed_delegation_id_1 UUID,
  signed_delegation_id_2 UUID,
  sender_address BYTEA,
  sender_reward_wei NUMERIC(78,0),
  burned_wei NUMERIC(78,0),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (operator_data_id) REFERENCES operator_data(id),
  FOREIGN KEY (signed_delegation_id_1) REFERENCES signed_delegation(id),
  FOREIGN KEY (signed_delegation_id_2) REFERENCES signed_delegation(id),
  FOREIGN KEY (sender_address, chain_id) REFERENCES address(address, chain_id),
  FOREIGN KEY (writer_id) REFERENCES writers(id)
);

SELECT trigger_audit_log('slash_equivocation');
SELECT trigger_updated_at('slash_equivocation');
