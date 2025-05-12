
CREATE TYPE "event_status" AS ENUM (
  'not_finalized',
  'finalized',
  'rolledback'
);


CREATE TABLE "events" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "block_number" int not null,
  "tx_hash" varchar,
  "status" event_status default ('not_finalized'),
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid,
  "config_id" uuid
);


ALTER TABLE "events" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");
ALTER TABLE "events" ADD FOREIGN KEY ("config_id") REFERENCES "config" ("id");

SELECT trigger_audit_log('events');
SELECT trigger_updated_at('events');
