use crate::utils::{advance_chain, setup};
use reth::blockchain_tree::error::BlockchainTreeError;
use reth_rpc_types::engine::PayloadStatusEnum;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn can_sync() -> eyre::Result<()> {
    // Initialize tracing for test logging
    reth_tracing::init_test_tracing();

    // Setup nodes, tasks, and wallet
    let (mut nodes, _tasks, wallet) = setup(3).await?;
    let wallet = Arc::new(Mutex::new(wallet));

    // Extract nodes for interaction
    let third_node = nodes.pop().unwrap();
    let mut second_node = nodes.pop().unwrap();
    let mut first_node = nodes.pop().unwrap();

    // Define tip block number and reorg depth
    let tip: usize = 90;
    let tip_index: usize = tip - 1;
    let reorg_depth = 2;

    // On first node, create a canonical chain up to block number 90a
    let canonical_payload_chain = advance_chain(tip, &mut first_node, wallet.clone()).await?;
    let canonical_chain =
        canonical_payload_chain.iter().map(|p| p.0.block().hash()).collect::<Vec<_>>();

    // On second node, sync optimistically up to block number 88a
    second_node
        .engine_api
        .update_optimistic_forkchoice(canonical_chain[tip_index - reorg_depth])
        .await?;
    second_node
        .wait_block((tip - reorg_depth) as u64, canonical_chain[tip_index - reorg_depth], true)
        .await?;

    // On third node, sync optimistically up to block number 90a
    third_node.engine_api.update_optimistic_forkchoice(canonical_chain[tip_index]).await?;
    third_node.wait_block(tip as u64, canonical_chain[tip_index], true).await?;

    // On second node, create a side chain: 88a -> 89b -> 90b
    wallet.lock().await.inner_nonce -= reorg_depth as u64;
    second_node.payload.timestamp = first_node.payload.timestamp - reorg_depth as u64;
    let side_payload_chain = advance_chain(reorg_depth, &mut second_node, wallet.clone()).await?;
    let side_chain = side_payload_chain.iter().map(|p| p.0.block().hash()).collect::<Vec<_>>();

    // Submit the 89b payload to third node, marking it as Valid
    let _ = third_node
        .engine_api
        .submit_payload(
            side_payload_chain[0].0.clone(),
            side_payload_chain[0].1.clone(),
            PayloadStatusEnum::Valid,
            Default::default(),
        )
        .await;

    // Update fork choice on third node to recognize 89b as canonical and finalized
    third_node.engine_api.update_forkchoice(side_chain[0], side_chain[0]).await?;

    // Ensure third node has updated to the reorged block
    third_node.wait_unwind((tip - reorg_depth) as u64).await?;
    third_node
        .wait_block(
            side_payload_chain[0].0.block().number,
            side_payload_chain[0].0.block().hash(),
            true,
        )
        .await?;

    // Attempt to submit 89a again to third node, expecting Invalid status due to 89b being finalized
    let _ = third_node
        .engine_api
        .submit_payload(
            canonical_payload_chain[tip_index - reorg_depth + 1].0.clone(),
            canonical_payload_chain[tip_index - reorg_depth + 1].1.clone(),
            PayloadStatusEnum::Invalid {
                validation_error: BlockchainTreeError::PendingBlockIsFinalized {
                    last_finalized: (tip - reorg_depth) as u64 + 1,
                }
                .to_string(),
            },
            Default::default(),
        )
        .await;

    Ok(())
}
