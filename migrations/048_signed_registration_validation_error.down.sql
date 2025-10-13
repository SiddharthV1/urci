DROP INDEX IF EXISTS idx_signed_registration_validation_error;

ALTER TABLE signed_registration
DROP COLUMN IF EXISTS validation_error;
