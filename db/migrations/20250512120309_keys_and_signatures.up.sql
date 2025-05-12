CREATE TYPE "keytype" AS ENUM (
  'BLS_PUB_KEY',
  'BLS_SIGNATURE',
  'ECDSA_PUB_KEY',
  'ECDSA_SIGNATURE'
);

CREATE TABLE "keys_and_signatures" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "event_id" uuid,
  "key_type" keytype,
  "g1_point_x_a" int,
  "g1_point_x_b" int,
  "g1_point_y_a" int,
  "g1_point_y_b" int,
  "g2_point_c0_x_a" int,
  "g2_point_c0_x_b" int,
  "g2_point_c0_y_a" int,
  "g2_point_c0_y_b" int,
  "g2_point_c1_x_a" int,
  "g2_point_c1_x_b" int,
  "g2_point_c1_y_a" int,
  "g2_point_c1_y_b" int,
  "bytes" bytea,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "writer_id" uuid
);

ALTER TABLE "keys_and_signatures" ADD FOREIGN KEY ("event_id") REFERENCES "events" ("id");

ALTER TABLE "keys_and_signatures" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

SELECT trigger_audit_log('keys_and_signatures');
SELECT trigger_updated_at('keys_and_signatures');
