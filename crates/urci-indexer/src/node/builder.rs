//! Node builder and orchestration
//!
//! Coordinates the 7-step initialization process:
//! 1. Initialize DB adapter
//! 2. Create BLS/Merkle EVM Executor
//! 3. Launch Node with ExEx
//! 4. Initialize Tracer
//! 5. Initialize TxPool Monitor
//! 6. Start indexer task
//! 7. Wait for shutdown

use eyre::Result;
use reth::cli::Cli;
use reth_node_ethereum::{EthereumAddOns, EthereumNode};
use std::sync::Arc;
use tracing::{error, info};
use urci_common::UrciConfig;
use urci_db_adapter::Db;

use super::db_init::initialize_database;
use super::tracer_init::initialize_tracer;
use super::txpool_monitor::{spawn_txpool_monitor, TaskSpawner};

// Implement TaskSpawner for the reth task executor
impl<T> TaskSpawner for T
where
    T: reth_tasks::TaskSpawner,
{
    fn spawn(&self, fut: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) {
        reth_tasks::TaskSpawner::spawn(self, fut);
    }
}

/// Build and launch the URCI indexer node
///
/// This orchestrates all initialization steps and starts the indexer
pub fn build_and_launch(mut config: UrciConfig) -> Result<()> {
    info!("Building URCI indexer node");
    info!("Registry address: {}", config.user.registry_address);
    info!("Database URL: {}", config.db.url);

    Cli::parse_args().run(|builder, _args| async move {
            let chain_spec = builder.config().chain.chain();
            let chain_id = chain_spec.id();

            config.fill_node_config(Some(chain_id), Some(format!("{:?}", chain_spec)));

            let chain_id_i64 = chain_id as i64;

            // Step 1: Initialize database
            info!("Step 1: Initializing database");
            let (mut db_adapter, writer_id) = initialize_database(
                chain_id_i64,
                &config.db.url,
                config.user.registry_address,
                config.user.backfill_from,
            )
            .await?;
            let tables_exist = db_adapter.is_initialized().await?;
            info!("✓ Database initialized");

            // Step 2: Create BLS/Merkle executor
            info!("Step 2: Launching BLS executor");
            let verifier = Arc::new(urci_bls_merkle::launch_bls_executor()?);
            info!("✓ BLS executor launched");

            // Step 3: Launch node with ExEx
            info!("Step 3: Launching node with ExEx");
            let (indexer_handle, indexer_rx) = urci_common::IndexerHandle::channel(1000);
            let exex_config = crate::exex::ExExConfig {
                registry_address: config.user.registry_address,
            };

            let verifier_for_exex = verifier.clone();
            let node_handle = builder
                .with_types::<EthereumNode>()
                .with_components(EthereumNode::components())
                .with_add_ons(EthereumAddOns::default())
                .install_exex("urci-indexer", move |ctx| {
                    let cfg = exex_config.clone();
                    let idx = indexer_handle.clone();
                    let ver = verifier_for_exex.clone();

                    Box::pin(async move {
                        info!("Initializing URCI ExEx");
                        crate::exex::init(ctx, cfg, idx, ver).await
                    })
                })
                .launch()
                .await?;
            info!("✓ Node launched");

            // Step 4: Initialize tracer and fetch registry config
            info!("Step 4: Initializing tracer");
            let eth_api = node_handle.node.rpc_registry.eth_api().clone();
            let (tracer, registry_config) = initialize_tracer(
                eth_api,
                config.user.registry_address,
                verifier.as_ref().clone(),
            )
            .await?;

            // Write registry config to DB if this is first initialization
            if !tables_exist {
                db_adapter
                    .write_config(writer_id, config.user.registry_address, registry_config)
                    .await?;
            }

            // Step 5: Spawn TxPool monitor
            info!("Step 5: Spawning txpool monitor");
            spawn_txpool_monitor(
                node_handle.node.pool.clone(),
                tracer.clone(),
                config.user.registry_address,
                node_handle.node.task_executor.clone(),
            );

            // Step 6: Spawn indexer task
            info!("Step 6: Spawning indexer task");
            let indexer_config = config.clone();
            let indexer_adapter = db_adapter;
            let indexer_tracer = tracer.clone();
            node_handle
                .node
                .task_executor
                .spawn(Box::pin(async move {
                    if let Err(e) =
                        crate::indexer::run_indexer(indexer_rx, indexer_adapter, indexer_config, indexer_tracer)
                            .await
                    {
                        error!("FATAL: Indexer task failed: {}", e);
                        error!(
                            "This is a critical error - the indexer will continue running \
                             but all writes will go to failed_block_writes table"
                        );
                    }
                }));
            info!("✓ Indexer task spawned");

            info!("🚀 URCI Indexer fully initialized and running");

            // Step 7: Wait for node shutdown
            node_handle.node_exit_future.await
        })
}
