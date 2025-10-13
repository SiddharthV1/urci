//! BlsOps trait implementation for EvmBlsMerkleExecutor

use alloy_primitives::U256;
use eyre::Result;
use urci_common::BLS;

use super::executor::EvmBlsMerkleExecutor;
use super::types::ContractType;
use crate::traits::BlsOps;
use crate::wrapper_contracts::{decode_bool, decode_g1_point, decode_g2_point, BLSWrapperCalls};

impl BlsOps for EvmBlsMerkleExecutor {
    fn to_public_key(&mut self, private_key: U256) -> Result<BLS::G1Point> {
        let calldata = BLSWrapperCalls::to_public_key(private_key);
        let result = self.execute_call(ContractType::Bls, calldata)?;
        decode_g1_point(&result)
    }

    fn to_message_point(
        &mut self,
        message: &[u8],
        domain_separator: &[u8],
    ) -> Result<BLS::G2Point> {
        let calldata = BLSWrapperCalls::to_message_point(message, domain_separator);
        let result = self.execute_call(ContractType::Bls, calldata)?;
        decode_g2_point(&result)
    }

    fn sign(
        &mut self,
        message: &[u8],
        private_key: U256,
        domain_separator: &[u8],
    ) -> Result<BLS::G2Point> {
        let calldata = BLSWrapperCalls::sign(message, private_key, domain_separator);
        let result = self.execute_call(ContractType::Bls, calldata)?;
        decode_g2_point(&result)
    }

    fn verify(
        &mut self,
        message: &[u8],
        signature: &BLS::G2Point,
        public_key: &BLS::G1Point,
        domain_separator: &[u8],
    ) -> Result<bool> {
        let calldata = BLSWrapperCalls::verify(message, signature, public_key, domain_separator);
        let result = self.execute_call(ContractType::Bls, calldata)?;
        decode_bool(&result)
    }
}
#[cfg(test)]
mod tests {
    //! Tests for BlsOps EVM implementation
    //!
    //! URC uses BLS signatures on the BN254 curve (also known as alt_bn128):
    //! - G1 points (public keys): 4 x 32-byte components (x_a, x_b, y_a, y_b)
    //! - G2 points (signatures, message points): 8 x 32-byte components
    //! - Domain separation prevents signature reuse across different protocols
    //! - URC registration uses abi.encode(operator_address) as the message

    use super::*;
    use crate::evm::EvmBlsMerkleExecutor;
    use alloy_primitives::{Address, U256};
    use alloy_sol_types::SolValue;

    // Test constants from UnitTestHelper.sol
    const SECRET_KEY_1: u64 = 12345;
    const SECRET_KEY_2: u64 = 67890;
    const REGISTRATION_DOMAIN: &[u8] = b"\x00URC";

    fn operator_address() -> Address {
        Address::from([0x42; 20])
    }

    /// Test that public key derivation matches known values from BLS.t.sol
    /// Uses test vector with private key 12356 from Solidity tests
    /// Verifies our EVM implementation produces identical results to the contract
    #[test]
    fn test_public_key_derivation_matches_solidity_test_vector() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key = U256::from(12356u64);
        let result = executor.to_public_key(private_key).unwrap();

        // Expected values from BLS.t.sol testToPublicKey()
        let x_a_expected = U256::from(12115118667309283734868789696201968385u128);
        assert_eq!(
            U256::from_be_bytes(result.x_a.0) & U256::from(u128::MAX),
            x_a_expected,
            "x_a component should match Solidity test vector"
        );

        let y_a_expected = U256::from(15699442850880472822588013448545136667u128);
        assert_eq!(
            U256::from_be_bytes(result.y_a.0) & U256::from(u128::MAX),
            y_a_expected,
            "y_a component should match Solidity test vector"
        );
    }

    /// Test that signature with wrong domain separator fails verification
    #[test]
    fn test_signature_with_wrong_domain_fails_verification() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key = U256::from(SECRET_KEY_1);
        let message = b"test message";
        let correct_domain = REGISTRATION_DOMAIN;
        let wrong_domain = b"\x00Wrong";

        let public_key = executor.to_public_key(private_key).unwrap();

        // Sign with correct domain
        let signature = executor.sign(message, private_key, correct_domain).unwrap();
        let is_valid = executor
            .verify(message, &signature, &public_key, correct_domain)
            .unwrap();
        assert!(is_valid, "Signature with correct domain should verify");

        // Sign with wrong domain
        let bad_signature = executor.sign(message, private_key, wrong_domain).unwrap();
        let invalid = executor
            .verify(message, &bad_signature, &public_key, correct_domain)
            .unwrap();
        assert!(
            !invalid,
            "Signature signed with wrong domain should not verify"
        );
    }

    /// Test URC registration & verification flow with abi.encode(operator_address) as message
    #[test]
    fn test_urc_registration_signature_with_abi_encoded_operator() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key = U256::from(SECRET_KEY_1);
        let operator = operator_address();
        let message = operator.abi_encode();

        let public_key = executor.to_public_key(private_key).unwrap();
        let signature = executor
            .sign(&message, private_key, REGISTRATION_DOMAIN)
            .unwrap();
        let is_valid = executor
            .verify(&message, &signature, &public_key, REGISTRATION_DOMAIN)
            .unwrap();

        assert!(is_valid, "Registration signature verification failed");
    }

    /// Test sign-verify cycle with various inputs (private keys, messages, domains)
    #[test]
    fn test_sign_verify_cycle_various_inputs() {
        // Based on BLS.t.sol testSignAndVerify fuzz test
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();

        let test_cases = vec![
            (1u64, b"message1".to_vec(), b"domain1".as_slice()),
            (u64::MAX, b"message2".to_vec(), b"domain2".as_slice()),
            (42, b"".to_vec(), b"".as_slice()),
            (
                SECRET_KEY_1,
                b"very long message that spans multiple words".to_vec(),
                REGISTRATION_DOMAIN,
            ),
        ];

        for (key, message, domain) in test_cases {
            let private_key = U256::from(key);
            let public_key = executor.to_public_key(private_key).unwrap();
            let signature = executor.sign(&message, private_key, domain).unwrap();
            let is_valid = executor
                .verify(&message, &signature, &public_key, domain)
                .unwrap();
            assert!(is_valid, "Signature verification failed for key {}", key);
        }
    }

    /// Test that signature verification fails when using wrong public key
    #[test]
    fn test_signature_with_wrong_key_fails_verification() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key_1 = U256::from(SECRET_KEY_1);
        let private_key_2 = U256::from(SECRET_KEY_2);
        let message = b"test message";

        let public_key_1 = executor.to_public_key(private_key_1).unwrap();
        let public_key_2 = executor.to_public_key(private_key_2).unwrap();

        // Sign with key 1, try to verify with key 2
        let signature = executor
            .sign(message, private_key_1, REGISTRATION_DOMAIN)
            .unwrap();
        let is_valid = executor
            .verify(message, &signature, &public_key_2, REGISTRATION_DOMAIN)
            .unwrap();
        assert!(
            !is_valid,
            "Signature from key 1 should not verify with key 2"
        );

        // Verify it works with the correct key
        let is_valid = executor
            .verify(message, &signature, &public_key_1, REGISTRATION_DOMAIN)
            .unwrap();
        assert!(is_valid, "Signature should verify with correct key");
    }

    /// Test that signature verification fails when message is modified
    #[test]
    fn test_signature_with_modified_message_fails() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key = U256::from(SECRET_KEY_1);
        let original_message = b"original message";
        let modified_message = b"modified message";

        let public_key = executor.to_public_key(private_key).unwrap();

        // Sign original message
        let signature = executor
            .sign(original_message, private_key, REGISTRATION_DOMAIN)
            .unwrap();

        // Try to verify with modified message
        let is_valid = executor
            .verify(
                modified_message,
                &signature,
                &public_key,
                REGISTRATION_DOMAIN,
            )
            .unwrap();
        assert!(!is_valid, "Signature should not verify modified message");

        // Verify it works with original message
        let is_valid = executor
            .verify(
                original_message,
                &signature,
                &public_key,
                REGISTRATION_DOMAIN,
            )
            .unwrap();
        assert!(is_valid, "Signature should verify original message");
    }

    /// Test that different keys produce independent, non-overlapping signatures
    #[test]
    fn test_cross_key_signatures_are_independent() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key_1 = U256::from(SECRET_KEY_1);
        let private_key_2 = U256::from(SECRET_KEY_2);
        let message = b"same message";

        let public_key_1 = executor.to_public_key(private_key_1).unwrap();
        let public_key_2 = executor.to_public_key(private_key_2).unwrap();

        // Sign same message with both keys
        let signature_1 = executor
            .sign(message, private_key_1, REGISTRATION_DOMAIN)
            .unwrap();
        let signature_2 = executor
            .sign(message, private_key_2, REGISTRATION_DOMAIN)
            .unwrap();

        // Signatures should be different
        assert!(
            signature_1.x_c0_a != signature_2.x_c0_a || signature_1.x_c0_b != signature_2.x_c0_b,
            "Different keys should produce different signatures"
        );

        // Each signature only verifies with its own key
        let valid_1_1 = executor
            .verify(message, &signature_1, &public_key_1, REGISTRATION_DOMAIN)
            .unwrap();
        let valid_1_2 = executor
            .verify(message, &signature_1, &public_key_2, REGISTRATION_DOMAIN)
            .unwrap();
        let valid_2_1 = executor
            .verify(message, &signature_2, &public_key_1, REGISTRATION_DOMAIN)
            .unwrap();
        let valid_2_2 = executor
            .verify(message, &signature_2, &public_key_2, REGISTRATION_DOMAIN)
            .unwrap();

        assert!(valid_1_1, "Signature 1 should verify with key 1");
        assert!(!valid_1_2, "Signature 1 should not verify with key 2");
        assert!(!valid_2_1, "Signature 2 should not verify with key 1");
        assert!(valid_2_2, "Signature 2 should verify with key 2");
    }

    /// Test that empty message can be signed and verified correctly
    #[test]
    fn test_empty_message_signature_works() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key = U256::from(SECRET_KEY_1);
        let empty_message = b"";

        let public_key = executor.to_public_key(private_key).unwrap();
        let signature = executor
            .sign(empty_message, private_key, REGISTRATION_DOMAIN)
            .unwrap();
        let is_valid = executor
            .verify(empty_message, &signature, &public_key, REGISTRATION_DOMAIN)
            .unwrap();

        assert!(is_valid, "Empty message signature should verify");

        // Verify signature is deterministic
        let signature_2 = executor
            .sign(empty_message, private_key, REGISTRATION_DOMAIN)
            .unwrap();
        assert_eq!(
            signature.x_c0_a, signature_2.x_c0_a,
            "Empty message signatures should be deterministic"
        );
    }

    /// Test that domain separator affects the signature
    #[test]
    fn test_domain_separator_affects_signature() {
        let mut executor = EvmBlsMerkleExecutor::new().unwrap();
        let private_key = U256::from(SECRET_KEY_1);
        let message = b"test message";
        let domain_1 = b"\x00URC";
        let domain_2 = b"\x00AVS";

        let public_key = executor.to_public_key(private_key).unwrap();

        // Sign same message with different domains
        let signature_1 = executor.sign(message, private_key, domain_1).unwrap();
        let signature_2 = executor.sign(message, private_key, domain_2).unwrap();

        // Signatures should be different
        assert!(
            signature_1.x_c0_a != signature_2.x_c0_a || signature_1.y_c0_a != signature_2.y_c0_a,
            "Different domains should produce different signatures"
        );

        // Each signature only verifies with its own domain
        let valid_1_1 = executor
            .verify(message, &signature_1, &public_key, domain_1)
            .unwrap();
        let valid_1_2 = executor
            .verify(message, &signature_1, &public_key, domain_2)
            .unwrap();
        let valid_2_1 = executor
            .verify(message, &signature_2, &public_key, domain_1)
            .unwrap();
        let valid_2_2 = executor
            .verify(message, &signature_2, &public_key, domain_2)
            .unwrap();

        assert!(valid_1_1, "Signature 1 should verify with domain 1");
        assert!(!valid_1_2, "Signature 1 should not verify with domain 2");
        assert!(!valid_2_1, "Signature 2 should not verify with domain 1");
        assert!(valid_2_2, "Signature 2 should verify with domain 2");
    }
}
