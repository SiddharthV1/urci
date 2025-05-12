
CREATE TYPE "verification_status" AS ENUM (
  'unverified',
  'in_progress',
  'verified',
  'failed_signature_verification',
  'failed_proof_verification'
);




CREATE TABLE "verifications" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "registration_id" uuid,
  "verification_status" verification_status,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "verifications" ADD FOREIGN KEY ("registration_id") REFERENCES "events" ("id");

ALTER TABLE "verifications" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('verifications');
SELECT trigger_updated_at('verifications');

