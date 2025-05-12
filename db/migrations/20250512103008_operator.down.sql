-- Add down migration script here
-- Drop the audit log trigger and the updated_at trigger
DROP TRIGGER IF EXISTS audit_log_trigger ON operators;
DROP TRIGGER IF EXISTS set_updated_at ON operators;
DROP TABLE IF EXISTS operators;

DROP TRIGGER IF EXISTS audit_log_trigger ON operator_collateral;
DROP TRIGGER IF EXISTS set_updated_at ON operator_collateral;
DROP TABLE IF EXISTS operator_collateral;

DROP TRIGGER IF EXISTS audit_log_trigger ON operator_record;
DROP TRIGGER IF EXISTS set_updated_at ON operator_record;
DROP TABLE IF EXISTS operator_record;
