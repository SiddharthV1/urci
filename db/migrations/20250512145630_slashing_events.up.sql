CREATE TYPE "slashing_type" AS ENUM (
  'fraud',
  'equivocation',
  'commitment'
);


CREATE TABLE "slashing_events" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "event_id" uuid,
  "operator_id" uuid,
  "slasher_id" uuid,
  "challenger" varchar,
  "slash_amount_wei" int,
  "slashing_type" slashing_type,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "slashing_events" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "slashing_events" ADD FOREIGN KEY ("operator_id") REFERENCES "operators" ("id");

ALTER TABLE "slashing_events" ADD FOREIGN KEY ("slasher_id") REFERENCES "slasher" ("id");

ALTER TABLE "slashing_events" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('slashing_events');
SELECT trigger_updated_at('slashing_events');
