CREATE TYPE "transfer_type" AS ENUM (
  'added',
  'claimed'
);

CREATE TABLE "collateral_transfer_events" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "event_id" uuid,
  "operator_id" uuid,
  "collateral" int,
  "transfer_type" transfer_type,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "collateral_transfer_events" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "collateral_transfer_events" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "collateral_transfer_events" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('collateral_transfer_events');
SELECT trigger_updated_at('collateral_transfer_events');

