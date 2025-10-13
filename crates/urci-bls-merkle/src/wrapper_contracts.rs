//! Wrapper contract interfaces for BLSUtils and MerkleTree
//!
//! These contracts expose library functions as external calls

use alloy_primitives::{Address, Bytes, B256, U256, FixedBytes};
use alloy_sol_types::{SolCall, SolValue, sol};
use eyre::{eyre, Result};
use urci_common::{IRegistry, BLS};

// We need to redefine the types in sol! because the macro can't reference external types
// But we'll convert to/from urc_common types in all our functions
sol! {
    // These structs match BLS types exactly - only used for ABI encoding/decoding
    #[derive(Debug)]
    struct G1Point {
        bytes32 x_a;
        bytes32 x_b;
        bytes32 y_a;
        bytes32 y_b;
    }
    
    #[derive(Debug)]
    struct G2Point {
        bytes32 x_c0_a;
        bytes32 x_c0_b;
        bytes32 x_c1_a;
        bytes32 x_c1_b;
        bytes32 y_c0_a;
        bytes32 y_c0_b;
        bytes32 y_c1_a;
        bytes32 y_c1_b;
    }
    
    #[derive(Debug)]
    struct SignedRegistration {
        G1Point pubkey;
        G2Point signature;
    }

    #[derive(Debug)]
    interface BLSUtilsWrapper {
        function toPublicKey(uint256 privateKey) external view returns (G1Point memory);

        function toMessagePoint(bytes memory message, bytes memory domainSeparator)
            external view returns (G2Point memory);

        function sign(bytes memory message, uint256 privateKey, bytes memory domainSeparator)
            external view returns (G2Point memory);

        function verify(
            bytes memory message,
            G2Point memory signature,
            G1Point memory publicKey,
            bytes memory domainSeparator
        ) external view returns (bool);

        function mulG1(G1Point memory point, bytes32 scalar)
            external view returns (G1Point memory);

        function mulG2(G2Point memory point, bytes32 scalar)
            external view returns (G2Point memory);
    }

    #[derive(Debug)]
    interface MerkleTreeWrapper {
        function generateTree(bytes32[] memory leaves) external pure returns (bytes32);

        function generateProof(bytes32[] memory leaves, uint256 index)
            external pure returns (bytes32[] memory);

        function verifyProof(bytes32 root, bytes32 leaf, bytes32[] memory proof)
            external pure returns (bool);

        function verifyProofCalldata(bytes32 root, bytes32 leaf, bytes32[] calldata proof)
            external pure returns (bool);

        function hashToLeaves(SignedRegistration[] calldata regs, address owner)
            external pure returns (bytes32[] memory);
    }
}

// The sol! macro generates the Call types we need

/// Load wrapper contract bytecode (after compilation)
pub fn load_bls_wrapper_bytecode() -> Result<Bytes> {
    // For now, return empty bytecode - will be replaced after compilation
    // In production, this would load from the compiled JSON
    Ok(Bytes::from(vec![0x60, 0x80, 0x60, 0x40])) // Minimal bytecode
}

/// Load wrapper contract bytecode (after compilation)
pub fn load_merkle_wrapper_bytecode() -> Result<Bytes> {
    // For now, return empty bytecode - will be replaced after compilation
    // In production, this would load from the compiled JSON
    Ok(Bytes::from(vec![0x60, 0x80, 0x60, 0x40])) // Minimal bytecode
}

/// Encode BLS wrapper calls
pub struct BLSWrapperCalls;

impl BLSWrapperCalls {
    pub fn to_public_key(private_key: U256) -> Bytes {
        let call = BLSUtilsWrapper::toPublicKeyCall { privateKey: private_key };
        Bytes::from(call.abi_encode())
    }

    pub fn to_message_point(message: &[u8], domain_separator: &[u8]) -> Bytes {
        let call = BLSUtilsWrapper::toMessagePointCall {
            message: Bytes::from(message.to_vec()),
            domainSeparator: Bytes::from(domain_separator.to_vec()),
        };
        Bytes::from(call.abi_encode())
    }

    pub fn sign(message: &[u8], private_key: U256, domain_separator: &[u8]) -> Bytes {
        let call = BLSUtilsWrapper::signCall {
            message: Bytes::from(message.to_vec()),
            privateKey: private_key,
            domainSeparator: Bytes::from(domain_separator.to_vec()),
        };
        Bytes::from(call.abi_encode())
    }

    pub fn verify(
        message: &[u8],
        signature: &BLS::G2Point,
        public_key: &BLS::G1Point,
        domain_separator: &[u8],
    ) -> Bytes {
        // Convert from urc_common types to local types
        // Note: BLS::G2Point has flattened Fp2 fields
        let sig = G2Point {
            x_c0_a: signature.x_c0_a,
            x_c0_b: signature.x_c0_b,
            x_c1_a: signature.x_c1_a,
            x_c1_b: signature.x_c1_b,
            y_c0_a: signature.y_c0_a,
            y_c0_b: signature.y_c0_b,
            y_c1_a: signature.y_c1_a,
            y_c1_b: signature.y_c1_b,
        };
        let pubkey = G1Point {
            x_a: public_key.x_a,
            x_b: public_key.x_b,
            y_a: public_key.y_a,
            y_b: public_key.y_b,
        };
        
        let call = BLSUtilsWrapper::verifyCall {
            message: Bytes::from(message.to_vec()),
            signature: sig,
            publicKey: pubkey,
            domainSeparator: Bytes::from(domain_separator.to_vec()),
        };
        Bytes::from(call.abi_encode())
    }
}

/// Encode Merkle wrapper calls
pub struct MerkleWrapperCalls;

impl MerkleWrapperCalls {
    pub fn generate_tree(leaves: &[B256]) -> Bytes {
        // Convert B256 to FixedBytes<32>
        let leaves_fixed: Vec<FixedBytes<32>> = leaves.iter()
            .map(|b| FixedBytes::from_slice(b.as_slice()))
            .collect();
        let call = MerkleTreeWrapper::generateTreeCall { leaves: leaves_fixed };
        Bytes::from(call.abi_encode())
    }

    pub fn generate_proof(leaves: &[B256], index: usize) -> Bytes {
        let leaves_fixed: Vec<FixedBytes<32>> = leaves.iter()
            .map(|b| FixedBytes::from_slice(b.as_slice()))
            .collect();
        let call = MerkleTreeWrapper::generateProofCall {
            leaves: leaves_fixed,
            index: U256::from(index),
        };
        Bytes::from(call.abi_encode())
    }

    pub fn verify_proof(root: B256, leaf: B256, proof: &[B256]) -> Bytes {
        let root_fixed = FixedBytes::from_slice(root.as_slice());
        let leaf_fixed = FixedBytes::from_slice(leaf.as_slice());
        let proof_fixed: Vec<FixedBytes<32>> = proof.iter()
            .map(|b| FixedBytes::from_slice(b.as_slice()))
            .collect();
        let call = MerkleTreeWrapper::verifyProofCall { root: root_fixed, leaf: leaf_fixed, proof: proof_fixed };
        Bytes::from(call.abi_encode())
    }

    pub fn hash_to_leaves(
        registrations: &[IRegistry::SignedRegistration],
        owner: Address,
    ) -> Bytes {
        // Convert IRegistry::SignedRegistration to local SignedRegistration
        let local_regs: Vec<SignedRegistration> = registrations.iter()
            .map(|r| SignedRegistration {
                pubkey: G1Point {
                    x_a: r.pubkey.x_a,
                    x_b: r.pubkey.x_b,
                    y_a: r.pubkey.y_a,
                    y_b: r.pubkey.y_b,
                },
                signature: G2Point {
                    x_c0_a: r.signature.x_c0_a,
                    x_c0_b: r.signature.x_c0_b,
                    x_c1_a: r.signature.x_c1_a,
                    x_c1_b: r.signature.x_c1_b,
                    y_c0_a: r.signature.y_c0_a,
                    y_c0_b: r.signature.y_c0_b,
                    y_c1_a: r.signature.y_c1_a,
                    y_c1_b: r.signature.y_c1_b,
                },
            })
            .collect();
        let call = MerkleTreeWrapper::hashToLeavesCall { regs: local_regs, owner };
        Bytes::from(call.abi_encode())
    }
}

/// Decode wrapper contract responses
pub fn decode_g1_point(data: &Bytes) -> Result<BLS::G1Point> {
    // Decode using the local G1Point type
    let local_point = G1Point::abi_decode(data)
        .map_err(|e| eyre!("Failed to decode G1Point: {}", e))?;
    
    // Convert to urc_common BLS::G1Point
    Ok(BLS::G1Point {
        x_a: local_point.x_a,
        x_b: local_point.x_b,
        y_a: local_point.y_a,
        y_b: local_point.y_b,
    })
}

pub fn decode_g2_point(data: &Bytes) -> Result<BLS::G2Point> {
    // Decode using the local G2Point type
    let local_point = G2Point::abi_decode(data)
        .map_err(|e| eyre!("Failed to decode G2Point: {}", e))?;
    
    // Convert to urc_common BLS::G2Point (with flattened Fp2 fields)
    Ok(BLS::G2Point {
        x_c0_a: local_point.x_c0_a,
        x_c0_b: local_point.x_c0_b,
        x_c1_a: local_point.x_c1_a,
        x_c1_b: local_point.x_c1_b,
        y_c0_a: local_point.y_c0_a,
        y_c0_b: local_point.y_c0_b,
        y_c1_a: local_point.y_c1_a,
        y_c1_b: local_point.y_c1_b,
    })
}

pub fn decode_bool(data: &Bytes) -> Result<bool> {
    bool::abi_decode(data).map_err(|e| eyre!("Failed to decode bool: {}", e))
}

pub fn decode_bytes32(data: &Bytes) -> Result<B256> {
    let fixed = FixedBytes::<32>::abi_decode(data)
        .map_err(|e| eyre!("Failed to decode bytes32: {}", e))?;
    Ok(B256::from_slice(fixed.as_slice()))
}

pub fn decode_bytes32_array(data: &Bytes) -> Result<Vec<B256>> {
    let fixed_array = Vec::<FixedBytes<32>>::abi_decode(data)
        .map_err(|e| eyre!("Failed to decode bytes32[]: {}", e))?;
    Ok(fixed_array.into_iter()
        .map(|f| B256::from_slice(f.as_slice()))
        .collect())
}
