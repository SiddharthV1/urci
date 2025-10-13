-- =============================================================================
-- BLS PUBKEY G1 POINT TABLE
-- =============================================================================

CREATE TABLE bls_pubkey_g1_point (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  json JSONB,
  bls_pubkey_g1_point_x_a BYTEA,
  bls_pubkey_g1_point_x_b BYTEA,
  bls_pubkey_g1_point_y_a BYTEA,
  bls_pubkey_g1_point_y_b BYTEA,
  bytes BYTEA,
  writer_id UUID,
  FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL
);
