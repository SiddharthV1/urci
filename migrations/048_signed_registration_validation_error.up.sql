-- =============================================================================
-- SIGNED REGISTRATION VALIDATION ERROR (Migration 017)
-- =============================================================================

-- Add validation_error column to signed_registration
ALTER TABLE signed_registration
ADD COLUMN IF NOT EXISTS validation_error TEXT;

-- Add index for querying registrations with validation errors
CREATE INDEX IF NOT EXISTS idx_signed_registration_validation_error
ON signed_registration (validation_error)
WHERE validation_error IS NOT NULL;

-- Comment on the column
COMMENT ON COLUMN signed_registration.validation_error IS 'Validation error message if signature verification failed. NULL means validation passed or was not performed yet.';
