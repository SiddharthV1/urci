-- =============================================================================
-- BLS SIGNATURE G2 POINT TABLE
-- =============================================================================

CREATE TABLE bls_signature_g2_point (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  json JSONB,
  bls_signature_g2_point_c0_x_a BYTEA,
  bls_signature_g2_point_c0_x_b BYTEA,
  bls_signature_g2_point_c0_y_a BYTEA,
  bls_signature_g2_point_c0_y_b BYTEA,
  bls_signature_g2_point_c1_x_a BYTEA,
  bls_signature_g2_point_c1_x_b BYTEA,
  bls_signature_g2_point_c1_y_a BYTEA,
  bls_signature_g2_point_c1_y_b BYTEA,
  bytes BYTEA,
  writer_id UUID,
  FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL
);
