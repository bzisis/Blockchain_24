use async_trait::async_trait;
use jsonrpsee::core::RpcResult as Result;
use reth_primitives::Address;
use reth_rpc_api::TxPoolApiServer;
use reth_rpc_types::{
    txpool::{TxpoolContent, TxpoolContentFrom, TxpoolInspect, TxpoolInspectSummary, TxpoolStatus},
    Transaction,
};
use reth_transaction_pool::{AllPoolTransactions, PoolTransaction, TransactionPool};
use std::collections::BTreeMap;
use tracing::trace;

/// `txpool` API implementation.
///
/// This type provides the functionality for handling `txpool` related requests.
#[derive(Clone)]
pub struct TxPoolApi<Pool> {
    /// An interface to interact with the pool
    pool: Pool,
}

impl<Pool> TxPoolApi<Pool> {
    /// Creates a new instance of `TxpoolApi`.
    ///
    /// # Arguments
    ///
    /// * `pool` - A transaction pool instance implementing the `TransactionPool` trait.
    ///
    /// # Returns
    ///
    /// A new instance of `TxPoolApi`.
    pub const fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

impl<Pool> TxPoolApi<Pool>
where
    Pool: TransactionPool + 'static,
{
    /// Retrieves the content of the transaction pool.
    ///
    /// This method extracts the pending and queued transactions from the transaction pool
    /// and organizes them in a `TxpoolContent` structure.
    ///
    /// # Returns
    ///
    /// A `TxpoolContent` structure containing the pending and queued transactions.
    fn content(&self) -> TxpoolContent {
        #[inline]
        fn insert<T: PoolTransaction>(
            tx: &T,
            content: &mut BTreeMap<Address, BTreeMap<String, Transaction>>,
        ) {
            content.entry(tx.sender()).or_default().insert(
                tx.nonce().to_string(),
                reth_rpc_types_compat::transaction::from_recovered(tx.to_recovered_transaction()),
            );
        }

        let AllPoolTransactions { pending, queued } = self.pool.all_transactions();

        let mut content = TxpoolContent::default();
        for pending in pending {
            insert(&pending.transaction, &mut content.pending);
        }
        for queued in queued {
            insert(&queued.transaction, &mut content.queued);
        }

        content
    }
}

#[async_trait]
impl<Pool> TxPoolApiServer for TxPoolApi<Pool>
where
    Pool: TransactionPool + 'static,
{
    /// Handler for `txpool_status`.
    ///
    /// Returns the number of transactions currently pending for inclusion in the next block(s), as
    /// well as the ones that are being scheduled for future execution only.
    async fn txpool_status(&self) -> Result<TxpoolStatus> {
        trace!(target: "rpc::eth", "Serving txpool_status");
        let all = self.pool.all_transactions();
        Ok(TxpoolStatus { pending: all.pending.len() as u64, queued: all.queued.len() as u64 })
    }

    /// Handler for `txpool_inspect`.
    ///
    /// Returns a summary of all the transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    async fn txpool_inspect(&self) -> Result<TxpoolInspect> {
        trace!(target: "rpc::eth", "Serving txpool_inspect");

        #[inline]
        fn insert<T: PoolTransaction>(
            tx: &T,
            inspect: &mut BTreeMap<Address, BTreeMap<String, TxpoolInspectSummary>>,
        ) {
            let entry = inspect.entry(tx.sender()).or_default();
            let tx = tx.to_recovered_transaction();
            entry.insert(
                tx.nonce().to_string(),
                TxpoolInspectSummary {
                    to: tx.to(),
                    value: tx.value(),
                    gas: tx.gas_limit() as u128,
                    gas_price: tx.transaction.max_fee_per_gas(),
                },
            );
        }

        let AllPoolTransactions { pending, queued } = self.pool.all_transactions();

        Ok(TxpoolInspect {
            pending: pending.iter().fold(Default::default(), |mut acc, tx| {
                insert(&tx.transaction, &mut acc);
                acc
            }),
            queued: queued.iter().fold(Default::default(), |mut acc, tx| {
                insert(&tx.transaction, &mut acc);
                acc
            }),
        })
    }

    /// Handler for `txpool_contentFrom`.
    ///
    /// Retrieves the transactions contained within the txpool, returning pending as well as queued
    /// transactions of this address, grouped by nonce.
    async fn txpool_content_from(&self, from: Address) -> Result<TxpoolContentFrom> {
        trace!(target: "rpc::eth", ?from, "Serving txpool_contentFrom");
        Ok(self.content().remove_from(&from))
    }

    /// Handler for `txpool_content`.
    ///
    /// Returns the details of all transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    async fn txpool_content(&self) -> Result<TxpoolContent> {
        trace!(target: "rpc::eth", "Serving txpool_content");
        Ok(self.content())
    }
}

impl<Pool> std::fmt::Debug for TxPoolApi<Pool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TxpoolApi").finish_non_exhaustive()
    }
}
