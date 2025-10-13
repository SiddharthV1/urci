//! MerkleOps trait implementation for EvmBlsMerkleExecutor

use alloy_primitives::{Address, B256};
use eyre::Result;
use urci_common::IRegistry;

use super::executor::EvmBlsMerkleExecutor;
use super::types::ContractType;
use crate::traits::MerkleOps;
use crate::wrapper_contracts::{
    decode_bool, decode_bytes32, decode_bytes32_array, MerkleWrapperCalls,
};

impl MerkleOps for EvmBlsMerkleExecutor {
    fn generate_tree(&mut self, leaves: &[B256]) -> Result<B256> {
        let calldata = MerkleWrapperCalls::generate_tree(leaves);
        let result = self.execute_call(ContractType::Merkle, calldata)?;
        decode_bytes32(&result)
    }

    fn generate_proof(&mut self, leaves: &[B256], index: usize) -> Result<Vec<B256>> {
        let calldata = MerkleWrapperCalls::generate_proof(leaves, index);
        let result = self.execute_call(ContractType::Merkle, calldata)?;
        decode_bytes32_array(&result)
    }

    fn verify_proof(&mut self, root: B256, leaf: B256, proof: &[B256]) -> Result<bool> {
        let calldata = MerkleWrapperCalls::verify_proof(root, leaf, proof);
        let result = self.execute_call(ContractType::Merkle, calldata)?;
        decode_bool(&result)
    }

    fn hash_to_leaves(
        &mut self,
        registrations: &[IRegistry::SignedRegistration],
        owner: Address,
    ) -> Result<Vec<B256>> {
        let calldata = MerkleWrapperCalls::hash_to_leaves(registrations, owner);
        let result = self.execute_call(ContractType::Merkle, calldata)?;
        decode_bytes32_array(&result)
    }
}

#[cfg(test)]
mod tests {
    //! Tests for MerkleOps EVM implementation
    //!
    //! The URC MerkleTree contract uses Solady's MerkleTreeLib which pads leaves
    //! to the next power of 2 with bytes32(0) fill. This ensures the tree is always
    //! a complete binary tree, making proof generation/verification simpler.
    //!
    //! Examples of padding behavior:
    //! - 1 leaf  → 1 (2⁰) - no padding
    //! - 2 leaves → 2 (2¹) - no padding
    //! - 3 leaves → 4 (2²) - padded with 1 zero
    //! - 5 leaves → 8 (2³) - padded with 3 zeros
    //! - 9 leaves → 16 (2⁴) - padded with 7 zeros

    use crate::evm::EvmBlsMerkleExecutor;
    use crate::traits::MerkleOps;
    use alloy_primitives::{hex, keccak256, Address, B256, FixedBytes, U256};
    use urci_common::{BLS, IRegistry};

    fn operator_address() -> Address {
        Address::from([0x42; 20])
    }

    /// Create dummy test registrations for merkle tree testing
    /// Each registration gets unique but deterministic dummy data based on its index
    fn create_test_registrations(count: usize) -> Vec<IRegistry::SignedRegistration> {
        let mut registrations = Vec::new();

        for i in 0..count {
            let registration = IRegistry::SignedRegistration {
                pubkey: BLS::G1Point {
                    x_a: FixedBytes::from([(i as u8) | 0x01; 32]),
                    x_b: FixedBytes::from([(i as u8) | 0x02; 32]),
                    y_a: FixedBytes::from([(i as u8) | 0x03; 32]),
                    y_b: FixedBytes::from([(i as u8) | 0x04; 32]),
                },
                signature: BLS::G2Point {
                    x_c0_a: FixedBytes::from([(i as u8) | 0x05; 32]),
                    x_c0_b: FixedBytes::from([(i as u8) | 0x06; 32]),
                    x_c1_a: FixedBytes::from([(i as u8) | 0x07; 32]),
                    x_c1_b: FixedBytes::from([(i as u8) | 0x08; 32]),
                    y_c0_a: FixedBytes::from([(i as u8) | 0x09; 32]),
                    y_c0_b: FixedBytes::from([(i as u8) | 0x0a; 32]),
                    y_c1_a: FixedBytes::from([(i as u8) | 0x0b; 32]),
                    y_c1_b: FixedBytes::from([(i as u8) | 0x0c; 32]),
                },
            };
            registrations.push(registration);
        }

        registrations
    }

    /// Test that a single leaf tree produces the leaf itself as root (no padding needed)
    /// Single leaf = 2⁰ = already a power of 2, so no padding occurs
    #[test]
    fn test_single_leaf_produces_itself_as_root() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let leaf1 = B256::from(hex!("0000000000000000000000000000000000000000000000000000000000000001"));
        let leaves = vec![leaf1];

        let root = executor.generate_tree(&leaves).unwrap();
        assert_eq!(root, leaf1, "Single leaf root should equal the leaf itself");

        let proof = executor.generate_proof(&leaves, 0).unwrap();
        assert_eq!(proof.len(), 0, "Single leaf has no siblings, so proof is empty");

        let is_valid = executor.verify_proof(root, leaf1, &proof).unwrap();
        assert!(is_valid, "Single leaf proof verification should succeed");
    }

    /// Test binary tree with 2 leaves (depth 1, no padding needed)
    /// 2 leaves = 2¹ = already a power of 2, tests both leaf proofs
    #[test]
    fn test_two_leaves_binary_tree_depth_1() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let leaf1 = keccak256(b"leaf1");
        let leaf2 = keccak256(b"leaf2");
        let leaves = vec![leaf1, leaf2];

        let root = executor.generate_tree(&leaves).unwrap();

        // Each leaf's proof contains its sibling (depth 1 = 1 sibling)
        let proof0 = executor.generate_proof(&leaves, 0).unwrap();
        assert_eq!(proof0.len(), 1, "Depth 1 tree has 1 sibling in proof");
        let is_valid0 = executor.verify_proof(root, leaf1, &proof0).unwrap();
        assert!(is_valid0, "Proof for leaf 0 should verify");

        let proof1 = executor.generate_proof(&leaves, 1).unwrap();
        assert_eq!(proof1.len(), 1, "Depth 1 tree has 1 sibling in proof");
        let is_valid1 = executor.verify_proof(root, leaf2, &proof1).unwrap();
        assert!(is_valid1, "Proof for leaf 1 should verify");
    }

    /// Test perfect binary tree with 4 leaves (depth 2, no padding needed)
    /// 4 leaves = 2² = already a power of 2, tests all leaves
    #[test]
    fn test_four_leaves_perfect_binary_tree_depth_2() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let leaves = vec![
            keccak256(b"leaf1"),
            keccak256(b"leaf2"),
            keccak256(b"leaf3"),
            keccak256(b"leaf4"),
        ];

        let root = executor.generate_tree(&leaves).unwrap();

        // Depth 2 tree: each leaf needs 2 siblings (one at each level)
        for i in 0..leaves.len() {
            let proof = executor.generate_proof(&leaves, i).unwrap();
            assert_eq!(proof.len(), 2, "Depth 2 tree has 2 siblings in proof");
            let is_valid = executor.verify_proof(root, leaves[i], &proof).unwrap();
            assert!(is_valid, "Proof for leaf {} should verify", i);
        }
    }

    /// Test 3 leaves gets padded to 4 with bytes32(0) fill (depth 2)
    /// 3 leaves → 4 leaves (2²), the 4th slot is filled with zeros
    /// This is a critical test for the Solady padding behavior
    #[test]
    fn test_three_leaves_pads_to_four_with_zero_fill() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let leaves = vec![
            keccak256(U256::from(0).to_be_bytes::<32>()),
            keccak256(U256::from(1).to_be_bytes::<32>()),
            keccak256(U256::from(2).to_be_bytes::<32>()),
        ];

        let root = executor.generate_tree(&leaves).unwrap();

        // Despite only 3 leaves, tree is depth 2 (padded to 4)
        for i in 0..leaves.len() {
            let proof = executor.generate_proof(&leaves, i).unwrap();
            assert_eq!(proof.len(), 2, "Padded to depth 2, so proof has 2 siblings");
            let is_valid = executor.verify_proof(root, leaves[i], &proof).unwrap();
            assert!(is_valid, "Proof for leaf {} should verify despite padding", i);
        }
    }

    /// Test 8 leaves perfect binary tree (depth 3, no padding needed)
    /// 8 leaves = 2³ = already a power of 2
    #[test]
    fn test_eight_leaves_perfect_binary_tree_depth_3() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let size = 8;
        let mut leaves = Vec::new();
        for i in 0..size {
            leaves.push(keccak256(U256::from(i).to_be_bytes::<32>()));
        }

        let root = executor.generate_tree(&leaves).unwrap();

        // Depth 3 tree: each leaf needs 3 siblings
        for i in 0..size {
            let proof = executor.generate_proof(&leaves, i).unwrap();
            assert_eq!(proof.len(), 3, "Depth 3 tree has 3 siblings in proof");
            let is_valid = executor.verify_proof(root, leaves[i], &proof).unwrap();
            assert!(is_valid, "Proof for leaf {} should verify", i);
        }
    }

    /// Test padding behavior across power-of-two boundaries
    /// Tests sizes: 3,4,5 (→4), 7,8,9 (→8) to verify correct padding
    #[test]
    fn test_padding_behavior_across_power_of_two_boundaries() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let sizes = vec![3, 4, 5, 7, 8, 9];

        for size in sizes {
            let mut leaves = Vec::new();
            for j in 0..size {
                leaves.push(keccak256(U256::from(j).to_be_bytes::<32>()));
            }

            let root = executor.generate_tree(&leaves).unwrap();

            // Test representative leaf indices
            let indices = vec![0, size / 2, size - 1];
            for idx in indices {
                let proof = executor.generate_proof(&leaves, idx).unwrap();
                let is_valid = executor.verify_proof(root, leaves[idx], &proof).unwrap();
                assert!(is_valid, "Tree with {} leaves failed verification at index {}", size, idx);
            }
        }
    }

    /// Test that different leaf sets produce different, deterministic roots
    /// Verifies merkle tree determinism and uniqueness properties
    #[test]
    fn test_different_leaves_produce_different_roots_deterministically() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let mut last_root = B256::ZERO;

        for i in 0..5 {
            let mut leaves = Vec::new();
            for j in 0..4 {
                let mut data = Vec::new();
                data.extend_from_slice(&U256::from(i).to_be_bytes::<32>());
                data.extend_from_slice(&U256::from(j).to_be_bytes::<32>());
                leaves.push(keccak256(&data));
            }

            let root = executor.generate_tree(&leaves).unwrap();

            if i > 0 {
                assert!(root != last_root, "Different leaves must produce different roots");
            }
            last_root = root;

            // Verify all proofs work correctly
            for j in 0..4 {
                let proof = executor.generate_proof(&leaves, j).unwrap();
                let is_valid = executor.verify_proof(root, leaves[j], &proof).unwrap();
                assert!(is_valid, "Tree {} proof for leaf {} should verify", i, j);
            }
        }
    }

    /// Test that 5 leaves get padded to 8 (2³)
    /// 5 leaves → 8 leaves, verifies depth 3 padding behavior
    #[test]
    fn test_five_leaves_pads_to_eight() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let leaves = vec![
            keccak256(U256::from(10).to_be_bytes::<32>()),
            keccak256(U256::from(11).to_be_bytes::<32>()),
            keccak256(U256::from(12).to_be_bytes::<32>()),
            keccak256(U256::from(13).to_be_bytes::<32>()),
            keccak256(U256::from(14).to_be_bytes::<32>()),
        ];

        let root = executor.generate_tree(&leaves).unwrap();

        // Despite only 5 leaves, tree is depth 3 (padded to 8)
        for i in 0..leaves.len() {
            let proof = executor.generate_proof(&leaves, i).unwrap();
            assert_eq!(proof.len(), 3, "Padded to depth 3, so proof has 3 siblings");
            let is_valid = executor.verify_proof(root, leaves[i], &proof).unwrap();
            assert!(is_valid, "Proof for leaf {} should verify despite padding to 8", i);
        }
    }

    // ===== Proof Verification Tests =====
    // These tests validate that proof verification correctly accepts valid proofs
    // and rejects invalid ones (tampered proofs, wrong leaf, etc.)

    /// Test that an invalid proof is correctly rejected
    /// Verifies proof verification detects tampering
    #[test]
    fn test_invalid_proof_rejected() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let leaves = vec![
            keccak256(b"leaf1"),
            keccak256(b"leaf2"),
            keccak256(b"leaf3"),
            keccak256(b"leaf4"),
        ];

        let root = executor.generate_tree(&leaves).unwrap();
        let proof = executor.generate_proof(&leaves, 0).unwrap();

        // Create a wrong proof by flipping a bit
        let mut wrong_proof = proof.clone();
        if !wrong_proof.is_empty() {
            wrong_proof[0] = B256::from([0xFF; 32]);
        }

        let is_valid = executor.verify_proof(root, leaves[0], &wrong_proof).unwrap();
        assert!(!is_valid, "Tampered proof should be rejected");
    }

    /// Test that a proof for one leaf doesn't verify a different leaf
    /// Verifies proof specificity - proof for leaf A cannot prove leaf B
    #[test]
    fn test_proof_for_different_leaf_fails() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let leaves = vec![
            keccak256(b"leaf1"),
            keccak256(b"leaf2"),
            keccak256(b"leaf3"),
            keccak256(b"leaf4"),
        ];

        let root = executor.generate_tree(&leaves).unwrap();
        let proof_for_leaf0 = executor.generate_proof(&leaves, 0).unwrap();

        // First verify the proof works with the correct leaf
        let is_valid = executor.verify_proof(root, leaves[0], &proof_for_leaf0).unwrap();
        assert!(is_valid, "Proof for leaf 0 should verify leaf 0");

        // Now try to use leaf0's proof to verify leaf1 (should fail)
        let is_valid = executor.verify_proof(root, leaves[1], &proof_for_leaf0).unwrap();
        assert!(!is_valid, "Proof for leaf 0 should not verify leaf 1");
    }

    // ===== URC Registration Tests =====
    // These tests validate hash_to_leaves() which converts SignedRegistration structs
    // into merkle tree leaves using keccak256(abi.encode(registration, owner))
    // and test the full URC registration merkle tree workflow

    /// Test that registration hashing matches Solidity's abi.encode behavior
    /// URC uses keccak256(abi.encode(registration, owner)) to create leaves
    #[test]
    fn test_registration_hashing_matches_solidity_abi_encode() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let operator = operator_address();

        let registrations = vec![IRegistry::SignedRegistration {
            pubkey: BLS::G1Point {
                x_a: FixedBytes::from([0x01; 32]),
                x_b: FixedBytes::from([0x02; 32]),
                y_a: FixedBytes::from([0x03; 32]),
                y_b: FixedBytes::from([0x04; 32]),
            },
            signature: BLS::G2Point {
                x_c0_a: FixedBytes::from([0x05; 32]),
                x_c0_b: FixedBytes::from([0x06; 32]),
                x_c1_a: FixedBytes::from([0x07; 32]),
                x_c1_b: FixedBytes::from([0x08; 32]),
                y_c0_a: FixedBytes::from([0x09; 32]),
                y_c0_b: FixedBytes::from([0x0a; 32]),
                y_c1_a: FixedBytes::from([0x0b; 32]),
                y_c1_b: FixedBytes::from([0x0c; 32]),
            },
        }];

        let leaves = executor.hash_to_leaves(&registrations, operator).unwrap();
        assert_eq!(leaves.len(), registrations.len(), "Should produce one leaf per registration");
        assert!(leaves[0] != B256::ZERO, "Leaf hash should not be zero (valid keccak256)");
    }

    /// Test hash_to_leaves with multiple registrations and full merkle workflow
    /// Tests 1, 2, 3 registrations to verify padding and proof generation
    /// Simulates the full URC registration merkle tree workflow
    #[test]
    fn test_hash_to_leaves_with_multiple_registrations() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let operator = operator_address();

        // Test cases: (description, registration_count, leaf_index_to_verify)
        let test_cases = vec![
            ("Single registration (depth 0)", 1, 0),
            ("Two registrations (depth 1)", 2, 0),
            ("Two registrations, second leaf (depth 1)", 2, 1),
            ("Three registrations, padded to 4 (depth 2)", 3, 0),
        ];

        for (description, count, leaf_index) in test_cases {
            // Generate test registrations with dummy data
            let registrations = create_test_registrations(count);

            // Hash registrations to merkle leaves
            let leaves = executor.hash_to_leaves(&registrations, operator).unwrap();
            assert_eq!(leaves.len(), count, "{}: Should produce one leaf per registration", description);

            // Generate merkle tree root
            let root = executor.generate_tree(&leaves).unwrap();
            assert!(root != B256::ZERO, "{}: Root should not be zero", description);

            // Generate merkle proof for specified leaf
            let proof = executor.generate_proof(&leaves, leaf_index).unwrap();

            // Verify the proof
            let is_valid = executor.verify_proof(root, leaves[leaf_index], &proof).unwrap();
            assert!(is_valid, "{}: Proof should verify", description);
        }
    }
}
