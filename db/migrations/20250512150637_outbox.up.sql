CREATE TYPE "outbox_status" AS ENUM (
  'pending',
  'dispatched',
  'processed',
  'error'
);

CREATE TABLE "outbox" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "aggregate_type" varchar(255) NOT NULL,
  "aggregate_id" varchar(255) NOT NULL,
  "event_type" varchar(255) NOT NULL,
  "payload" jsonb NOT NULL,
  "created_at" timestamp DEFAULT (now()),
  "dispatched_at" timestamp,
  "writer_id" uuid,
  "status" outbox_status DEFAULT ('pending')
);


ALTER TABLE "outbox" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");
