-- =============================================================================
-- OPERATOR COLLATERAL TABLE
-- =============================================================================

CREATE TABLE operator_collateral (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  registration_root BYTEA NOT NULL,
  chain_id INTEGER NOT NULL,
  collateral_wei_total NUMERIC(78,0),
  collateral_wei_delta NUMERIC(78,0),
  event_id BIGINT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (event_id) REFERENCES events(id),
  FOREIGN KEY (chain_id, registration_root) REFERENCES operators(chain_id, registration_root),
  FOREIGN KEY (writer_id) REFERENCES writers(id)
);

SELECT trigger_audit_log('operator_collateral');
SELECT trigger_updated_at('operator_collateral');
