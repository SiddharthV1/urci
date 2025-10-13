-- Drop triggers first
DROP TRIGGER IF EXISTS audit_log_trigger ON bls_signature_domains;
DROP TRIGGER IF EXISTS set_updated_at_trigger ON bls_signature_domains;

-- Drop table
DROP TABLE IF EXISTS bls_signature_domains CASCADE;
