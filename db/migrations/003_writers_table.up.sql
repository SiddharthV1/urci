-- =============================================================================
-- WRITERS TABLE
-- =============================================================================

-- Function to derive writer_id from config JSON by hashing
CREATE FUNCTION derive_writer_id(config JSONB)
RETURNS UUID
LANGUAGE SQL IMMUTABLE PARALLEL SAFE
AS $$
    SELECT uuid_generate_v5(
        '6ba7b810-9dad-11d1-80b4-00c04fd430c8'::uuid,
        digest(config::text, 'sha256')::text
    );
$$;

-- Writers table for multi-writer support
CREATE TABLE writers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    session TEXT NOT NULL UNIQUE,
    config JSONB NOT NULL,
    chain_id BIGINT GENERATED ALWAYS AS ((config->>'chain_id')::BIGINT) STORED,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for writers table
CREATE INDEX idx_writers_session ON writers(session);
