CREATE TABLE "proofs" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "operator_id" uuid,
  "bytes_32" bytea,
  "event_id" uuid,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "proofs" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "proofs" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "proofs" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('proofs');
SELECT trigger_updated_at('proofs');
