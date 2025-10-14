//! Clean URCI ExEx - minimal streaming design
//!
//! This module handles the ExEx (Execution Extension) for URCI indexing.
//! It receives blockchain notifications from reth and streams them to the indexer.

mod config;
mod consensus;
mod notification_handler;

pub use config::ExExConfig;

use eyre::Result;
use futures::Future;
use reth_exex::ExExContext;
use reth_node_api::{FullNodeComponents, NodeTypes};
use reth_primitives::EthPrimitives;
use std::sync::Arc;
use tracing::info;

use notification_handler::run_exex;

/// Main ExEx initialization function
///
/// Called by the node builder to install the URCI ExEx.
pub async fn init<Node, V>(
    ctx: ExExContext<Node>,
    config: ExExConfig,
    indexer: impl urci_common::Indexer + 'static,
    verifier: Arc<V>,
) -> Result<impl Future<Output = Result<()>>>
where
    Node: FullNodeComponents<Types: NodeTypes<Primitives = EthPrimitives>>,
    V: urci_common::Verifier + 'static,
{
    info!("Starting URCI ExEx (streaming mode)");
    info!("Registry: {:?}", config.registry_address);

    Ok(run_exex(ctx, config.registry_address, indexer, verifier))
}
