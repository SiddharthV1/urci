-- =============================================================================
-- SIGNED REGISTRATION TABLE
-- =============================================================================

CREATE TABLE signed_registration (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_address BYTEA NOT NULL,
  chain_id BIGINT NOT NULL DEFAULT 1,
  bls_signature_id UUID,
  bls_pub_key_id UUID,
  merkle_tree_id BYTEA,
  merkle_inclusion_proof_id UUID,
  verification_status verification_status DEFAULT 'unverified',
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  writer_id UUID,
  event_id BIGINT,
  FOREIGN KEY (owner_address, chain_id) REFERENCES address(address, chain_id),
  FOREIGN KEY (bls_signature_id) REFERENCES bls_signature_g2_point(id),
  FOREIGN KEY (bls_pub_key_id) REFERENCES bls_pubkey_g1_point(id),
  FOREIGN KEY (merkle_tree_id) REFERENCES merkle_tree(root),
  FOREIGN KEY (merkle_inclusion_proof_id) REFERENCES merkle_inclusion_proof(id),
  FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL,
  FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);

SELECT trigger_audit_log('signed_registration');
SELECT trigger_updated_at('signed_registration');
