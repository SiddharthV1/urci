-- Label providers registry
CREATE TABLE "label_providers" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "name" varchar(255) UNIQUE NOT NULL, -- e.g., 'etherscan', 'ens', 'dune', 'nansen'
  "description" text,
  "api_endpoint" varchar(512),
  "priority" int DEFAULT 0, -- Higher priority providers are queried first
  "is_active" boolean DEFAULT true,
  "rate_limit" int DEFAULT 100, -- Requests per minute
  "config" jsonb, -- Provider-specific configuration
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now())
);

-- Labels for addresses, transactions, and other entities
CREATE TABLE "labels" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "entity_type" varchar(50) NOT NULL, -- 'address', 'transaction', 'contract', 'token'
  "entity_id" varchar(255) NOT NULL, -- Address hash, tx hash, etc.
  "chain_id" int NOT NULL,
  "label" varchar(255) NOT NULL, -- The actual label text
  "label_type" varchar(50), -- 'exchange', 'protocol', 'scam', 'verified', 'whale', etc.
  "provider_id" uuid NOT NULL,
  "confidence_score" decimal(3,2) DEFAULT 1.00, -- 0.00 to 1.00
  "metadata" jsonb, -- Additional structured data
  "verified" boolean DEFAULT false,
  "created_at" timestamp DEFAULT (now()),
  "updated_at" timestamp DEFAULT (now()),
  "expires_at" timestamp, -- For temporary labels
  UNIQUE(entity_type, entity_id, chain_id, label, provider_id)
);

-- Label categories for organization
CREATE TABLE "label_categories" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "name" varchar(100) UNIQUE NOT NULL,
  "description" text,
  "color" varchar(7), -- Hex color for UI
  "icon" varchar(50),
  "parent_category_id" uuid,
  "created_at" timestamp DEFAULT (now())
);

-- Many-to-many relationship between labels and categories
CREATE TABLE "label_category_mappings" (
  "label_id" uuid NOT NULL,
  "category_id" uuid NOT NULL,
  PRIMARY KEY (label_id, category_id)
);

-- Label history for tracking changes
CREATE TABLE "label_history" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "label_id" uuid NOT NULL,
  "action" varchar(20) NOT NULL, -- 'created', 'updated', 'verified', 'expired'
  "old_value" jsonb,
  "new_value" jsonb,
  "changed_by" varchar(255), -- System or user identifier
  "created_at" timestamp DEFAULT (now())
);

-- Cache for expensive label queries
CREATE TABLE "label_cache" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "cache_key" varchar(512) UNIQUE NOT NULL,
  "provider_id" uuid NOT NULL,
  "response_data" jsonb NOT NULL,
  "ttl_seconds" int DEFAULT 3600,
  "created_at" timestamp DEFAULT (now()),
  "expires_at" timestamp NOT NULL
);

-- Label enrichment queue for async processing
CREATE TABLE "label_enrichment_queue" (
  "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
  "entity_type" varchar(50) NOT NULL,
  "entity_id" varchar(255) NOT NULL,
  "chain_id" int NOT NULL,
  "providers" text[], -- Array of provider names to query
  "priority" int DEFAULT 0,
  "status" varchar(20) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed'
  "attempts" int DEFAULT 0,
  "max_attempts" int DEFAULT 3,
  "last_error" text,
  "created_at" timestamp DEFAULT (now()),
  "processed_at" timestamp
);

-- Indexes for performance
CREATE INDEX idx_labels_entity ON labels(entity_type, entity_id, chain_id);
CREATE INDEX idx_labels_provider ON labels(provider_id);
CREATE INDEX idx_labels_type ON labels(label_type);
CREATE INDEX idx_labels_created ON labels(created_at DESC);
CREATE INDEX idx_label_cache_expires ON label_cache(expires_at);
CREATE INDEX idx_enrichment_queue_status ON label_enrichment_queue(status, priority DESC);

-- Foreign key constraints
ALTER TABLE "labels" ADD FOREIGN KEY ("provider_id") REFERENCES "label_providers" ("id");
ALTER TABLE "label_category_mappings" ADD FOREIGN KEY ("label_id") REFERENCES "labels" ("id") ON DELETE CASCADE;
ALTER TABLE "label_category_mappings" ADD FOREIGN KEY ("category_id") REFERENCES "label_categories" ("id");
ALTER TABLE "label_categories" ADD FOREIGN KEY ("parent_category_id") REFERENCES "label_categories" ("id");
ALTER TABLE "label_history" ADD FOREIGN KEY ("label_id") REFERENCES "labels" ("id") ON DELETE CASCADE;
ALTER TABLE "label_cache" ADD FOREIGN KEY ("provider_id") REFERENCES "label_providers" ("id");

-- Triggers
SELECT trigger_updated_at('label_providers');
SELECT trigger_updated_at('labels');
SELECT trigger_audit_log('labels');