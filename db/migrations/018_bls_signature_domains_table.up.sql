-- =============================================================================
-- BLS SIGNATURE DOMAINS TABLE
-- =============================================================================

CREATE TABLE bls_signature_domains (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  domain_type TEXT,
  name TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

SELECT trigger_audit_log('bls_signature_domains');
SELECT trigger_updated_at('bls_signature_domains');

INSERT INTO bls_signature_domains (id, domain_type, name) VALUES
  ('00000000-0000-0000-0000-000000000000', '0x00435255', 'REGISTRATION_DOMAIN_SEPARATOR'),
  ('00000000-0000-0000-0000-000000000001', '0x0044656c', 'DELEGATION_DOMAIN_SEPARATOR');
