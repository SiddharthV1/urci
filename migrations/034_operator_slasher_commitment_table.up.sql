-- =============================================================================
-- OPERATOR SLASHER COMMITMENT TABLE
-- =============================================================================

CREATE TABLE operator_slasher_commitment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  registration_root BYTEA NOT NULL,
  chain_id INTEGER NOT NULL,
  slasher_id UUID,
  committer_id UUID,
  opted_in_at TIMESTAMPTZ,
  opted_out_at TIMESTAMPTZ,
  slashed BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (chain_id, registration_root) REFERENCES operators(chain_id, registration_root),
  FOREIGN KEY (slasher_id) REFERENCES slasher(id),
  FOREIGN KEY (committer_id) REFERENCES committer(id),
  FOREIGN KEY (writer_id) REFERENCES writers(id)
);

SELECT trigger_audit_log('operator_slasher_commitment');
SELECT trigger_updated_at('operator_slasher_commitment');
