//! Some payload tests
//! 
//! This module contains tests for various aspects of payload transformation and validation.

use alloy_rlp::{Decodable, Error as RlpError};
use assert_matches::assert_matches;
use reth_primitives::{
    proofs, Block, Bytes, SealedBlock, TransactionSigned, Withdrawals, B256, U256,
};
use reth_rpc_types::engine::{
    ExecutionPayload, ExecutionPayloadBodyV1, ExecutionPayloadV1, PayloadError,
};
use reth_rpc_types_compat::engine::payload::{
    block_to_payload, block_to_payload_v1, convert_to_payload_body_v1, try_into_sealed_block,
    try_payload_v1_to_block,
};
use reth_testing_utils::generators::{self, random_block, random_block_range, random_header, Rng};

/// Transforms a `SealedBlock` by applying a function `f` to it and recalculating its roots.
/// 
/// # Arguments
/// 
/// * `src` - The original sealed block.
/// * `f` - A function that takes a `Block` and returns a transformed `Block`.
/// 
/// # Returns
/// 
/// An `ExecutionPayload` derived from the transformed block.
fn transform_block<F: FnOnce(Block) -> Block>(src: SealedBlock, f: F) -> ExecutionPayload {
    let unsealed = src.unseal();
    let mut transformed: Block = f(unsealed);
    // Recalculate roots
    transformed.header.transactions_root = proofs::calculate_transaction_root(&transformed.body);
    transformed.header.ommers_hash = proofs::calculate_ommers_root(&transformed.ommers);
    block_to_payload(SealedBlock {
        header: transformed.header.seal_slow(),
        body: transformed.body,
        ommers: transformed.ommers,
        withdrawals: transformed.withdrawals,
        requests: transformed.requests,
    })
    .0
}

/// Tests roundtrip conversion between block bodies and payload bodies.
#[test]
fn payload_body_roundtrip() {
    let mut rng = generators::rng();
    for block in random_block_range(&mut rng, 0..=99, B256::default(), 0..2) {
        let unsealed = block.clone().unseal();
        let payload_body: ExecutionPayloadBodyV1 = convert_to_payload_body_v1(unsealed);

        // Check that converting to payload body and back yields the original transactions
        assert_eq!(
            Ok(block.body),
            payload_body
                .transactions
                .iter()
                .map(|x| TransactionSigned::decode(&mut &x[..]))
                .collect::<Result<Vec<_>, _>>(),
        );
        
        // Check withdrawals
        let withdraw = payload_body.withdrawals.map(Withdrawals::new);
        assert_eq!(block.withdrawals, withdraw);
    }
}

/// Tests various aspects of payload validation.
#[test]
fn payload_validation() {
    let mut rng = generators::rng();
    let parent = rng.gen();
    let block = random_block(&mut rng, 100, Some(parent), Some(3), Some(0));

    // Valid extra data
    let block_with_valid_extra_data = transform_block(block.clone(), |mut b| {
        b.header.extra_data = Bytes::from_static(&[0; 32]);
        b
    });

    // Validate that the block with valid extra data is correctly transformed into a sealed block
    assert_matches!(try_into_sealed_block(block_with_valid_extra_data, None), Ok(_));

    // Invalid extra data
    let block_with_invalid_extra_data = Bytes::from_static(&[0; 33]);
    let invalid_extra_data_block = transform_block(block.clone(), |mut b| {
        b.header.extra_data = block_with_invalid_extra_data.clone();
        b
    });

    // Validate that the block with invalid extra data returns the expected error
    assert_matches!(
        try_into_sealed_block(invalid_extra_data_block, None),
        Err(PayloadError::ExtraData(data)) if data == block_with_invalid_extra_data
    );

    // Zero base fee
    let block_with_zero_base_fee = transform_block(block.clone(), |mut b| {
        b.header.base_fee_per_gas = Some(0);
        b
    });

    // Validate that the block with zero base fee returns the expected error
    assert_matches!(
        try_into_sealed_block(block_with_zero_base_fee, None),
        Err(PayloadError::BaseFee(val)) if val.is_zero()
    );

    // Invalid encoded transactions
    let mut payload_with_invalid_txs: ExecutionPayloadV1 = block_to_payload_v1(block.clone());

    // Invalidate all transactions in the payload
    payload_with_invalid_txs.transactions.iter_mut().for_each(|tx| {
        *tx = Bytes::new();
    });

    // Validate that the payload with invalid transactions returns the expected error
    let payload_with_invalid_txs = try_payload_v1_to_block(payload_with_invalid_txs);
    assert_matches!(payload_with_invalid_txs, Err(PayloadError::Decode(RlpError::InputTooShort)));

    // Non empty ommers
    let block_with_ommers = transform_block(block.clone(), |mut b| {
        b.ommers.push(random_header(&mut rng, 100, None).unseal());
        b
    });

    // Validate that the block with non-empty ommers returns the expected error
    assert_matches!(
        try_into_sealed_block(block_with_ommers.clone(), None),
        Err(PayloadError::BlockHash { consensus, .. })
            if consensus == block_with_ommers.block_hash()
    );

    // Non-zero difficulty
    let block_with_difficulty = transform_block(block.clone(), |mut b| {
        b.header.difficulty = U256::from(1);
        b
    });

    // Validate that the block with non-zero difficulty returns the expected error
    assert_matches!(
        try_into_sealed_block(block_with_difficulty.clone(), None),
        Err(PayloadError::BlockHash { consensus, .. }) if consensus == block_with_difficulty.block_hash()
    );

    // Non-zero nonce
    let block_with_nonce = transform_block(block.clone(), |mut b| {
        b.header.nonce = 1;
        b
    });

    // Validate that the block with non-zero nonce returns the expected error
    assert_matches!(
        try_into_sealed_block(block_with_nonce.clone(), None),
        Err(PayloadError::BlockHash { consensus, .. }) if consensus == block_with_nonce.block_hash()
    );

    // Valid block
    let valid_block = block;
    // Validate that a valid block is correctly transformed into a sealed block
    assert_matches!(TryInto::<SealedBlock>::try_into(valid_block), Ok(_));
}
