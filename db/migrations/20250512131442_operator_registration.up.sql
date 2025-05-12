CREATE TABLE "operator_registrations" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "event_id" uuid,
  "operator_id" uuid,
  "bls_pub_key" uuid,
  "bls_signature" uuid,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "operator_registrations" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "operator_registrations" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "operator_registrations" ADD FOREIGN KEY ("bls_pub_key") REFERENCES "keys_and_signatures" ("id");

ALTER TABLE "operator_registrations" ADD FOREIGN KEY ("bls_signature") REFERENCES "keys_and_signatures" ("id");

ALTER TABLE "operator_registrations" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('operator_registrations');
SELECT trigger_updated_at('operator_registrations');

