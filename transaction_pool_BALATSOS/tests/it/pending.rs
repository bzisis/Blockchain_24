// Author: Zisis Balatsos

///
/// This module contains integration tests for the transaction pool. These tests verify the behavior of the transaction pool, ensuring that 
/// transactions are correctly added and processed. The tests use mock transactions to simulate different scenarios.
///

// Tests:
// This test verifies the addition of new pending transactions to the transaction pool.
//
// The test performs the following steps:
// 
// 1. Creates a new transaction pool using `TestPoolBuilder`.
// 2. Creates a mock EIP-1559 transaction using `MockTransactionFactory`.
// 3. Adds the transaction to the pool and checks that it is successfully added.
// 4. Retrieves the best transactions from the pool and verifies that the transaction is correctly included.
// 5. Adds another mock EIP-1559 transaction and performs the same checks.
// 
// The assertions ensure that the transaction pool correctly handles the addition and retrieval of transactions.


use assert_matches::assert_matches;
use reth_transaction_pool::{
    test_utils::{MockTransactionFactory, TestPoolBuilder},
    TransactionOrigin, TransactionPool,
};

#[tokio::test(flavor = "multi_thread")]
async fn txpool_new_pending_txs() {
    // create a new transaction pool using the default configuration provided by "TestPoolBuilder"
    let txpool = TestPoolBuilder::default();

    // create a mock transaction factory which is used to generate mock transactions
    let mut mock_tx_factory = MockTransactionFactory::default();

    // create a mock EIP-1559 transaction using the factoryy
    let transaction = mock_tx_factory.create_eip1559();

    // the transaction is added to the pool, and the result is checked to ensure it was added successfully
    let added_result =
        txpool.add_transaction(TransactionOrigin::External, transaction.transaction.clone()).await;
    assert_matches!(added_result, Ok(hash) if hash == transaction.transaction.get_hash());

    // The best transactions are retrieved from the pool, and the first transaction is checked to ensure it matches the added transaction
    // The next transaction is checked to be None, indicating that there is only one transaction in the pool
    let mut best_txns = txpool.best_transactions();
    assert_matches!(best_txns.next(), Some(tx) if tx.transaction.get_hash() == transaction.transaction.get_hash());
    assert_matches!(best_txns.next(), None);

    // Another mock EIP-1559 transaction is generated
    let transaction = mock_tx_factory.create_eip1559();

    // The second transaction is added to the pool, and the result is checked to ensure it was added successfully
    let added_result =
        txpool.add_transaction(TransactionOrigin::External, transaction.transaction.clone()).await;

    // The first transaction in the best transactions is checked again to ensure it matches the newly added transaction
    assert_matches!(added_result, Ok(hash) if hash == transaction.transaction.get_hash());
    assert_matches!(best_txns.next(), Some(tx) if tx.transaction.get_hash() == transaction.transaction.get_hash());
}
