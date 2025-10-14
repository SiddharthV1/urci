//! Consensus finality data queries

use alloy_eips::BlockNumHash;
use reth_exex::ExExContext;
use reth_node_api::{FullNodeComponents, NodeTypes};
use reth_primitives::EthPrimitives;
use reth_storage_api::{BlockIdReader, BlockNumReader};

/// Query consensus finality data from the provider
///
/// Returns (finalized_block, safe_block, head_block)
pub fn query_consensus_data<Node>(
    ctx: &ExExContext<Node>,
    tip: BlockNumHash,
) -> (Option<BlockNumHash>, Option<BlockNumHash>, BlockNumHash)
where
    Node: FullNodeComponents<Types: NodeTypes<Primitives = EthPrimitives>>,
{
    let finalized = ctx
        .provider()
        .finalized_block_num_hash()
        .ok()
        .flatten();

    let safe = ctx
        .provider()
        .safe_block_num_hash()
        .ok()
        .flatten();

    let head = ctx
        .provider()
        .chain_info()
        .ok()
        .map(|info| BlockNumHash::new(info.best_number, info.best_hash))
        .unwrap_or(tip);

    (finalized, safe, head)
}
