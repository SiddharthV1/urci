// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "urc/lib/MerkleTree.sol";
import "urc/IRegistry.sol";

/**
 * @title MerkleTreeWrapper
 * @notice Wrapper contract to expose MerkleTree library functions as external calls
 * @dev Deploy this in genesis to enable EVM-based Merkle operations
 */
contract MerkleTreeWrapper {
    using MerkleTree for *;
    
    function generateTree(bytes32[] memory leaves) external pure returns (bytes32) {
        return MerkleTree.generateTree(leaves);
    }
    
    function generateProof(bytes32[] memory leaves, uint256 index) 
        external pure returns (bytes32[] memory) {
        return MerkleTree.generateProof(leaves, index);
    }
    
    function verifyProof(bytes32 root, bytes32 leaf, bytes32[] memory proof) 
        external pure returns (bool) {
        return MerkleTree.verifyProof(root, leaf, proof);
    }
    
    function verifyProofCalldata(bytes32 root, bytes32 leaf, bytes32[] calldata proof) 
        external pure returns (bool) {
        return MerkleTree.verifyProofCalldata(root, leaf, proof);
    }
    
    function hashToLeaves(IRegistry.SignedRegistration[] calldata regs, address owner)
        external pure returns (bytes32[] memory) {
        return MerkleTree.hashToLeaves(regs, owner);
    }
}