-- =============================================================================
-- AUDIT LOG TABLE
-- =============================================================================

-- Audit log table for change tracking
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(10) NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    table_name VARCHAR(63) NOT NULL,
    record_id TEXT NOT NULL,
    old_data JSONB,
    new_data JSONB,
    writer_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (writer_id) REFERENCES writers(id) ON DELETE SET NULL
);

-- Indexes for audit_log
CREATE INDEX idx_audit_log_table_record ON audit_log(table_name, record_id);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at DESC);
