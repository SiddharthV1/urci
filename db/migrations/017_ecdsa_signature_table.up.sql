-- =============================================================================
-- ECDSA SIGNATURE TABLE
-- =============================================================================

CREATE TABLE ecdsa_signature (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  r BYTEA,
  s BYTEA,
  v INTEGER,
  signature BYTEA,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL
);
