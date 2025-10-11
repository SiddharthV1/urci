-- =============================================================================
-- ADD DEFERRED FOREIGN KEY TO MERKLE_INCLUSION_PROOF
-- =============================================================================

-- Add the foreign key constraint that was deferred from merkle_inclusion_proof
ALTER TABLE merkle_inclusion_proof
ADD FOREIGN KEY (chain_id, registration_root) REFERENCES operators(chain_id, registration_root);
