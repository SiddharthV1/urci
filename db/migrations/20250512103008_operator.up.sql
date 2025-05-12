-- Add up migration script here

CREATE TABLE "operators" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "registration_root" varchar UNIQUE not null,
  "address" CHAR(42),
  "num_keys" int,
  "registration_processed" bool,
  "event_id" uuid,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "operators" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");
ALTER TABLE "operators" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('operators');
SELECT trigger_updated_at('operators');

CREATE TABLE "operator_collateral" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "operator_id" uuid,
  "collateral_wei" int,
  "event_id" uuid,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);

ALTER TABLE "operator_collateral" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "operator_collateral" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "operator_collateral" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('operator_collateral');
SELECT trigger_updated_at('operator_collateral');

CREATE TABLE "operator_record" (
  "operator_id" uuid,
  "unregistered_at" uuid,
  "registered_at" uuid,
  "slashed_at" int,
  "deleted" bool,
  "equivocated" bool,
  "event_id" uuid,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "operator_record" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "operator_record" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "operator_record" ADD FOREIGN KEY ("unregistered_at") REFERENCES "events" ("id");

ALTER TABLE "operator_record" ADD FOREIGN KEY ("registered_at") REFERENCES "events" ("id");

ALTER TABLE "operator_record" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('operator_record');
SELECT trigger_updated_at('operator_record');

