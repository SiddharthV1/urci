-- =============================================================================
-- OPERATORS TABLE
-- =============================================================================

CREATE TABLE operators (
  registration_root BYTEA NOT NULL,
  chain_id INTEGER NOT NULL,
  owner_address BYTEA NOT NULL,
  num_keys INTEGER,
  registration_processed BOOLEAN DEFAULT FALSE,
  event_id BIGINT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  PRIMARY KEY (chain_id, registration_root),
  -- Note: Same owner CAN register multiple times with different registration_roots
  -- UNIQUE (chain_id, owner_address) constraint intentionally omitted
  FOREIGN KEY (owner_address, chain_id) REFERENCES address(address, chain_id),
  FOREIGN KEY (registration_root) REFERENCES merkle_tree(root),
  FOREIGN KEY (event_id) REFERENCES events(id),
  FOREIGN KEY (writer_id) REFERENCES writers(id)
);

COMMENT ON TABLE operators IS
'Stores operator registrations. Same owner can have multiple registrations (different registration_roots). PRIMARY KEY enforces uniqueness of registration_root per chain, matching the Registry contract mapping.';

SELECT trigger_audit_log('operators');
SELECT trigger_updated_at('operators');
