-- Add up migration script here

CREATE TABLE "operators" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "registration_root" varchar UNIQUE not null,
  "address" CHAR(42),
  "num_keys" int,
  "registration_processed" bool,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);

SELECT trigger_audit_log('operators');
SELECT trigger_updated_at('operators');

CREATE TABLE "operator_collateral" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "operator_id" uuid,
  "collateral_wei" int,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);

SELECT trigger_audit_log('operator_collateral');
SELECT trigger_updated_at('operator_collateral');

CREATE TABLE "operator_record" (
  "operator_id" uuid,
  "unregistered_at" int,
  "registered_at" int,
  "slashed_at" int,
  "deleted" bool,
  "equivocated" bool,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


SELECT trigger_audit_log('operator_record');
SELECT trigger_updated_at('operator_record');

