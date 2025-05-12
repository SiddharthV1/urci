CREATE TABLE "config" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "contract_address" CHAR(42),
  "min_collateral_wei" int NOT NULL,
  "fraud_proof_window" int NOT NULL,
  "unregistration_delay" int NOT NULL,
  "slash_window" int NOT NULL,
  "opt_in_delay" int NOT NULL,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);

ALTER TABLE "config" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");


SELECT trigger_audit_log('config');
SELECT trigger_updated_at('config');
