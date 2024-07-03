// Author: Zisis Balatsos

// Contents:
// 1) The module-level doc comment "Blob transaction tests" -> provides an overview of the purpose of this test module
// 2) The doc comment for the "blobs_exclusive" function -> explains the steps and checks performed within the test
// 3) Inline comments within the function -> provide further explanation of each step, making the test logic clear and easy to understand
//

//! Blob transaction tests

/// 
/// This module contains tests to ensure the correct behavior of the transaction pool
/// when dealing with blob transactions, particularly ensuring that they are handled
/// exclusively and do not conflict with other transaction types.
///
use reth_transaction_pool::{
    error::PoolErrorKind,
    test_utils::{MockTransaction, MockTransactionFactory, TestPoolBuilder},
    TransactionOrigin, TransactionPool,
};

/// This test ensures that blob transactions are handled exclusively in the transaction pool.
/// It performs the following checks:
/// 1. Adds a blob transaction to the pool and verifies its inclusion.
/// 2. Ensures that the blob transaction is the only transaction in the pool.
/// 3. Attempts to add a conflicting EIP-1559 transaction and verifies that it is rejected
///    with the appropriate error indicating a conflict with the existing blob transaction.
///
#[tokio::test(flavor = "multi_thread")]
async fn blobs_exclusive() {
    // Create a test transaction pool
    let txpool = TestPoolBuilder::default();

    // Create a mock transaction factory
    let mut mock_tx_factory = MockTransactionFactory::default();

    // Create a blob transaction using the mock transaction factory
    let blob_tx = mock_tx_factory.create_eip4844();

    // Add the blob transaction to the transaction pool
    let hash = txpool
        .add_transaction(TransactionOrigin::External, blob_tx.transaction.clone())
        .await
        .unwrap();
    
    // Verify that the transaction hash matches the expected hash of the blob transaction
    assert_eq!(hash, blob_tx.transaction.get_hash());

    // Retrieve the best transactions from the pool and verify that the blob transaction is the only one
    let mut best_txns = txpool.best_transactions();
    assert_eq!(best_txns.next().unwrap().transaction.get_hash(), blob_tx.transaction.get_hash());
    assert!(best_txns.next().is_none());

    // Create a conflicting EIP-1559 transaction with the same sender as the blob transaction but with a higher price
    let eip1559_tx = MockTransaction::eip1559()
        .set_sender(blob_tx.transaction.get_sender())
        .inc_price_by(10_000);

    // Attempt to add the conflicting EIP-1559 transaction to the pool and verify that it is rejected
    let res =
        txpool.add_transaction(TransactionOrigin::External, eip1559_tx.clone()).await.unwrap_err();
    
    // Verify that the rejected transaction hash matches the expected hash of the EIP-1559 transaction
    assert_eq!(res.hash, eip1559_tx.get_hash());

    // Check that the error kind is `ExistingConflictingTransactionType` and verify the sender and transaction type
    match res.kind {
        PoolErrorKind::ExistingConflictingTransactionType(addr, tx_type) => {
            assert_eq!(addr, eip1559_tx.get_sender());
            assert_eq!(tx_type, eip1559_tx.tx_type());
        }
        _ => unreachable!(),
    }
}
