//! Build script to compile BLS/Merkle wrapper contracts AND URC contracts at build time
//!
//! Compiles contracts with proper library linking so the JSON artifacts
//! have linked bytecode (no placeholders).

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=contracts/BLSUtilsWrapper.sol");
    println!("cargo:rerun-if-changed=contracts/MerkleTreeWrapper.sol");
    println!("cargo:rerun-if-changed=contracts/foundry.toml");

    // Also watch the URC contracts submodule
    println!("cargo:rerun-if-changed=../../contracts/urc/src/Registry.sol");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let contracts_dir = manifest_dir.join("contracts");
    let urc_contracts_dir = manifest_dir.join("../../contracts/urc");

    // Check if forge is available
    let forge_available = Command::new("forge")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !forge_available {
        eprintln!("Warning: forge not found. Skipping contract compilation.");
        eprintln!("Contracts must be pre-compiled with forge build");
        return;
    }

    // Step 1: Compile wrapper contracts (BLS/Merkle with library linking)
    eprintln!("Compiling wrapper contracts with forge and library linking...");
    let status = Command::new("forge")
        .arg("build")
        .arg("--libraries")
        .arg("urc/lib/BLSUtils.sol:BLSUtils:0x000000000000000000000000000000000000B156")
        .current_dir(&contracts_dir)
        .status()
        .expect("Failed to run forge build for wrapper contracts");

    if !status.success() {
        panic!("forge build failed for wrapper contracts");
    }

    eprintln!("✓ Wrapper contracts compiled");

    // Step 2: Compile URC contracts (Registry, etc.)
    eprintln!("Compiling URC contracts...");
    let status = Command::new("forge")
        .arg("build")
        .current_dir(&urc_contracts_dir)
        .status()
        .expect("Failed to run forge build for URC contracts");

    if !status.success() {
        panic!("forge build failed for URC contracts");
    }

    eprintln!("✓ URC contracts compiled");

    // Step 3: Extract ABI-only JSON for Registry (to avoid unlinked bytecode issues)
    extract_abi_only(
        &urc_contracts_dir.join("out/Registry.sol/Registry.json"),
        &manifest_dir.join("registry_abi.json"),
    );

    eprintln!("✓ All contracts compiled - JSON artifacts ready");
}

/// Extract just the ABI from a full contract JSON artifact
fn extract_abi_only(full_json_path: &std::path::Path, abi_only_path: &std::path::Path) {
    use std::fs;

    if !full_json_path.exists() {
        eprintln!("Warning: {} not found, skipping ABI extraction", full_json_path.display());
        return;
    }

    let content = fs::read_to_string(full_json_path)
        .expect("Failed to read Registry.json");

    // Parse the JSON and extract just the ABI field
    let full_json: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse Registry.json");

    let abi = full_json.get("abi")
        .expect("No 'abi' field in Registry.json");

    // Write ABI-only JSON
    fs::write(abi_only_path, serde_json::to_string_pretty(abi).unwrap())
        .expect("Failed to write registry_abi.json");

    eprintln!("✓ Extracted ABI-only JSON: {}", abi_only_path.display());
}
