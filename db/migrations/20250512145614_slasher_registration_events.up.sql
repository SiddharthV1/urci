
CREATE TYPE "slasher_commitment_status" AS ENUM (
  'opt_in',
  'opt_out'
);

CREATE TABLE "slasher_registration_events" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "event_id" uuid,
  "operator_id" uuid,
  "slasher_id" uuid,
  "commitment_id" uuid,
  "status" slasher_commitment_status,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "slasher_registration_events" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "slasher_registration_events" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "slasher_registration_events" ADD FOREIGN KEY ("slasher_id") REFERENCES "slasher" ("id");

ALTER TABLE "slasher_registration_events" ADD FOREIGN KEY ("commitment_id") REFERENCES "committer" ("id");

ALTER TABLE "slasher_registration_events" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('slasher_registration_events');
SELECT trigger_updated_at('slasher_registration_events');
