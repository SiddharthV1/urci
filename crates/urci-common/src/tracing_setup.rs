use eyre::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Initialize tracing for tests
#[cfg(test)]
pub fn init_test_tracing() {
    let _ = init_tracing_with_filter(
        "urc=debug,urc_bls_merkle=debug,urc_common=debug,urc_exex=debug,revm=debug,reth_evm=debug,reth_revm=debug"
    );
}

// Initialize tracing for production with configurable filter
pub fn init_tracing() {
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "urc=info,revm=warn,reth_evm=warn".to_string());

    let _ = init_tracing_with_filter(&filter);
}

// Initialize tracing with a specific filter string
//
// - "urc=debug" - Debug all URC crates
// - "urc_bls_merkle=trace" - Trace BLS merkle operations
// - "revm=trace" - Trace all REVM operations
// - "revm::interpreter=trace" - Trace REVM interpreter specifically
// - "reth_evm=debug" - Debug reth EVM operations
pub fn init_tracing_with_filter(filter: &str) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::from(filter));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| eyre::eyre!("Failed to init tracing: {}", e))?;

    Ok(())
}
