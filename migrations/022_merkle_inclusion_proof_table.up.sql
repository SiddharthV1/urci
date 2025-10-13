-- =============================================================================
-- MERKLE INCLUSION PROOF TABLE
-- =============================================================================

CREATE TABLE merkle_inclusion_proof (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  merkle_index INTEGER NOT NULL,
  merkle_inclusion_proof BYTEA[],
  registration_root BYTEA,
  chain_id INTEGER,
  bytes_32 BYTEA,
  event_id BIGINT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (registration_root) REFERENCES merkle_tree(root),
  FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
  FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL
);

SELECT trigger_audit_log('merkle_inclusion_proof');
SELECT trigger_updated_at('merkle_inclusion_proof');
