-- =============================================================================
-- VERIFICATION STATUS TYPE
-- =============================================================================

CREATE TYPE verification_status AS ENUM (
  'unverified',
  'in_progress',
  'verified',
  'failed_signature_verification',
  'failed_proof_verification'
);
