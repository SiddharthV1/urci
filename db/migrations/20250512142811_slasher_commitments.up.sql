CREATE TABLE "slasher_commitment" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "commiter_id" uuid,
  "opted_in_at" timestamp,
  "opted_out_at" timestamp,
  "slashed" bool,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "slasher_commitment" ADD FOREIGN KEY ("commiter_id") REFERENCES "commiter" ("id");

ALTER TABLE "slasher_commitment" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('slasher_commitment');
SELECT trigger_updated_at('slasher_commitment');

CREATE TABLE "operator_slasher_commitments" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "operator_id" uuid,
  "slasher_id" uuid,
  "slasher_commitment_id" uuid,
  "opted_in_at" timestamp,
  "opted_out_at" timestamp,
  "slashed" bool,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
  -- PRIMARY KEY ("operator_id", "slasher_id", "slasher_commitment_id", "writer_id")
);


ALTER TABLE "operator_slasher_commitments" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "operator_slasher_commitments" ADD FOREIGN KEY ("slasher_id") REFERENCES "slasher" ("id");

ALTER TABLE "operator_slasher_commitments" ADD FOREIGN KEY ("slasher_commitment_id") REFERENCES "commiter" ("id");

ALTER TABLE "operator_slasher_commitments" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('operator_slasher_commitments');
SELECT trigger_updated_at('operator_slasher_commitments');
