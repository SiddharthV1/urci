-- =============================================================================
-- COMMITTER TABLE
-- =============================================================================

CREATE TABLE committer (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  chain_id INTEGER NOT NULL,
  address BYTEA NOT NULL,
  event_id BIGINT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (address, chain_id) REFERENCES address(address, chain_id),
  FOREIGN KEY (event_id) REFERENCES events(id),
  FOREIGN KEY (writer_id) REFERENCES writers(id)
);

SELECT trigger_audit_log('committer');
SELECT trigger_updated_at('committer');
