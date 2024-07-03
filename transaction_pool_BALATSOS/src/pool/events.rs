// Author: Zisis Balatsos

// Contents:
// 1) crate::{traits::PropagateKind, PoolTransaction, ValidPoolTransaction}: -> Importing types and traits from the local crate
// 2) reth_primitives::{TxHash, B256}: -> Importing TxHash and B256 from reth_primitives
// 3) std::sync::Arc: -> Importing Arc from the standard library for atomic reference counting
// 4) serde::{Deserialize, Serialize}: -> Conditionally importing Deserialize and Serialize from serde if the "serde" feature is enabled
// 5) Pending(TxHash) -> Indicates the transaction has been added to the pending pool
// 6) Pending(TxHash) -> Indicates the transaction has been added to the pending pool
// 7) Mined -> Indicates the transaction has been included in a block:
//      - tx_hash: The hash of the mined transaction
//      - block_hash: The hash of the block containing the transaction
// 8) Replaced -> Indicates the transaction has been replaced by another transaction:
//      - transaction: The transaction that was replaced, wrapped in Arc<ValidPoolTransaction<T>>
//      - replaced_by: The hash of the replacing transaction
// 9) Discarded(TxHash) -> Indicates the transaction was dropped due to configured limits
// 10) Invalid(TxHash) -> Indicates the transaction became invalid indefinitely
// 11) Propagated(Arc<Vec<PropagateKind>>) -> Indicates the transaction was propagated to peers, wrapped in Arc


use crate::{traits::PropagateKind, PoolTransaction, ValidPoolTransaction};
use reth_primitives::{TxHash, B256};
use std::sync::Arc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An event that happened to a transaction and contains its full body where possible.
#[derive(Debug)]
pub enum FullTransactionEvent<T: PoolTransaction> {
    /// Transaction has been added to the pending pool.
    Pending(TxHash),
    /// Transaction has been added to the queued pool.
    Queued(TxHash),
    /// Transaction has been included in the block belonging to this hash.
    Mined {
        /// The hash of the mined transaction.
        tx_hash: TxHash,
        /// The hash of the mined block that contains the transaction.
        block_hash: B256,
    },
    /// Transaction has been replaced by the transaction belonging to the hash.
    ///
    /// E.g. same (sender + nonce) pair
    Replaced {
        /// The transaction that was replaced.
        transaction: Arc<ValidPoolTransaction<T>>,
        /// The transaction that replaced the event subject.
        replaced_by: TxHash,
    },
    /// Transaction was dropped due to configured limits.
    Discarded(TxHash),
    /// Transaction became invalid indefinitely.
    Invalid(TxHash),
    /// Transaction was propagated to peers.
    Propagated(Arc<Vec<PropagateKind>>),
}

impl<T: PoolTransaction> Clone for FullTransactionEvent<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Pending(hash) => Self::Pending(*hash),
            Self::Queued(hash) => Self::Queued(*hash),
            Self::Mined { tx_hash, block_hash } => {
                Self::Mined { tx_hash: *tx_hash, block_hash: *block_hash }
            }
            Self::Replaced { transaction, replaced_by } => {
                Self::Replaced { transaction: Arc::clone(transaction), replaced_by: *replaced_by }
            }
            Self::Discarded(hash) => Self::Discarded(*hash),
            Self::Invalid(hash) => Self::Invalid(*hash),
            Self::Propagated(propagated) => Self::Propagated(Arc::clone(propagated)),
        }
    }
}

/// Various events that describe status changes of a transaction.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TransactionEvent {
    /// Transaction has been added to the pending pool.
    Pending,
    /// Transaction has been added to the queued pool.
    Queued,
    /// Transaction has been included in the block belonging to this hash.
    Mined(B256),
    /// Transaction has been replaced by the transaction belonging to the hash.
    ///
    /// E.g. same (sender + nonce) pair
    Replaced(TxHash),
    /// Transaction was dropped due to configured limits.
    Discarded,
    /// Transaction became invalid indefinitely.
    Invalid,
    /// Transaction was propagated to peers.
    Propagated(Arc<Vec<PropagateKind>>),
}

impl TransactionEvent {
    /// Returns `true` if the event is final and no more events are expected for this transaction
    /// hash.
    pub const fn is_final(&self) -> bool {
        matches!(self, Self::Replaced(_) | Self::Mined(_) | Self::Discarded)
    }
}
