-- Remove the foreign key constraint to operators
ALTER TABLE merkle_inclusion_proof
DROP CONSTRAINT IF EXISTS merkle_inclusion_proof_chain_id_registration_root_fkey;
