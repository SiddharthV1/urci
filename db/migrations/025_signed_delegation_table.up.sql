-- =============================================================================
-- SIGNED DELEGATION TABLE
-- =============================================================================

CREATE TABLE signed_delegation (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	chain_id INTEGER NOT NULL,
	proposer UUID,
	delegate UUID,
	committer BYTEA,
	slot BIGINT,
	metadata BYTEA,
	writer_id UUID,
	event_id BIGINT,
	message JSONB,
	signature UUID,
	created_at TIMESTAMPTZ DEFAULT NOW(),
	updated_at TIMESTAMPTZ DEFAULT NOW(),
	FOREIGN KEY (committer, chain_id) REFERENCES address(address, chain_id),
	FOREIGN KEY (proposer) REFERENCES bls_pubkey_g1_point(id),
	FOREIGN KEY (delegate) REFERENCES bls_pubkey_g1_point(id),
	FOREIGN KEY (signature) REFERENCES bls_signature_g2_point(id),
	FOREIGN KEY (writer_id) REFERENCES writers(id),
	FOREIGN KEY (event_id) REFERENCES events(id)
);

SELECT trigger_audit_log('signed_delegation');
SELECT trigger_updated_at('signed_delegation');
