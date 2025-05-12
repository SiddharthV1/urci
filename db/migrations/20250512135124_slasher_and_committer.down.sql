CREATE TABLE "slasher" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "address" CHAR(42),
  "event_id" uuid,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);

ALTER TABLE "slasher" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "slasher" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

select trigger_audit_log('slasher');
select trigger_updated_at('slasher');

CREATE TABLE "committer" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "address" CHAR(42),
  "event_id" uuid,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);


ALTER TABLE "committer" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "committer" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

select trigger_audit_log('committer');
select trigger_updated_at('committer');
