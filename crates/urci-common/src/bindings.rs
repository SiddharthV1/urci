use alloy_sol_types::sol;

// Solady BLS library
sol!(
    #[sol(all_derives)]
    BLS,
    "contracts/out/BLS.sol/BLS.json"
);

// MerkleTree library
sol!(
    #[sol(all_derives)]
    MerkleTree,
    "contracts/out/MerkleTree.sol/MerkleTree.json"
);
