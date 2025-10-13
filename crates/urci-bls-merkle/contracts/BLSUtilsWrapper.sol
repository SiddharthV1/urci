// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "urc/lib/BLSUtils.sol";
import { BLS } from "solady/utils/ext/ithaca/BLS.sol";

/**
 * @title BLSUtilsWrapper
 * @notice Wrapper contract to expose BLSUtils library functions as external calls
 * @dev Deploy this in genesis to enable EVM-based BLS operations
 */
contract BLSUtilsWrapper {
    using BLSUtils for *;
    
    function toPublicKey(uint256 privateKey) external view returns (BLS.G1Point memory) {
        return BLSUtils.toPublicKey(privateKey);
    }
    
    function toMessagePoint(bytes memory message, bytes memory domainSeparator) 
        external view returns (BLS.G2Point memory) {
        return BLSUtils.toMessagePoint(message, domainSeparator);
    }
    
    function sign(bytes memory message, uint256 privateKey, bytes memory domainSeparator)
        external view returns (BLS.G2Point memory) {
        return BLSUtils.sign(message, privateKey, domainSeparator);
    }
    
    function verify(
        bytes memory message,
        BLS.G2Point memory signature,
        BLS.G1Point memory publicKey,
        bytes memory domainSeparator
    ) external view returns (bool) {
        return BLSUtils.verify(message, signature, publicKey, domainSeparator);
    }
    
    function mulG1(BLS.G1Point memory point, bytes32 scalar) 
        external view returns (BLS.G1Point memory) {
        return BLSUtils.mul(point, scalar);
    }
    
    function mulG2(BLS.G2Point memory point, bytes32 scalar)
        external view returns (BLS.G2Point memory) {
        return BLSUtils.mul(point, scalar);
    }
}