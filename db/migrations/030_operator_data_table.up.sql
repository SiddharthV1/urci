-- =============================================================================
-- OPERATOR DATA TABLE
-- =============================================================================

CREATE TABLE operator_data (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  collateral_id UUID,
  registration_root BYTEA NOT NULL,
  chain_id INTEGER NOT NULL,
  unregistered_at BIGINT,
  registered_at BIGINT,
  slashed_at INTEGER,
  deleted BOOLEAN,
  equivocated BOOLEAN,
  event_id BIGINT,
  event JSONB,
  event_type operator_event_type,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (event_id) REFERENCES events(id),
  FOREIGN KEY (collateral_id) REFERENCES operator_collateral(id),
  FOREIGN KEY (chain_id, registration_root) REFERENCES operators(chain_id, registration_root),
  FOREIGN KEY (unregistered_at) REFERENCES events(id),
  FOREIGN KEY (registered_at) REFERENCES events(id),
  FOREIGN KEY (writer_id) REFERENCES writers(id)
);

SELECT trigger_audit_log('operator_data');
SELECT trigger_updated_at('operator_data');
