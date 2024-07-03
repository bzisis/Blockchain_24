// Author: Zisis Balatsos

use crate::blobstore::{BlobStore, BlobStoreCleanupStat, BlobStoreError, BlobTransactionSidecar};
use reth_primitives::B256;

/// A blobstore implementation that does nothing
/// This is a no-operation (noop) implementation of the `BlobStore` trait. It is used in scenarios
/// where a blob store is required but no actual storage operations are needed or desired.
/// All methods in this implementation do nothing and return default values where applicable.
///
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct NoopBlobStore;

impl BlobStore for NoopBlobStore {
    /// Inserts the blob sidecar into the store.
    ///
    /// This method does nothing and always returns `Ok(())`.
    ///
    /// # Arguments
    /// * `_tx` - The transaction hash (ignored).
    /// * `_data` - The blob data to be stored (ignored).
    ///
    fn insert(&self, _tx: B256, _data: BlobTransactionSidecar) -> Result<(), BlobStoreError> {
        Ok(())
    }

    /// Inserts multiple blob sidecars into the store.
    ///
    /// This method does nothing and always returns `Ok(())`.
    ///
    /// # Arguments
    /// * `_txs` - A vector of tuples containing the transaction hash and corresponding blob data (ignored).
    ///
    fn insert_all(&self, _txs: Vec<(B256, BlobTransactionSidecar)>) -> Result<(), BlobStoreError> {
        Ok(())
    }

    /// Deletes the blob sidecar from the store.
    ///
    /// This method does nothing and always returns `Ok(())`.
    ///
    /// # Arguments
    /// * `_tx` - The transaction hash to be deleted (ignored).
    ///
    fn delete(&self, _tx: B256) -> Result<(), BlobStoreError> {
        Ok(())
    }

    /// Deletes multiple blob sidecars from the store.
    ///
    /// This method does nothing and always returns `Ok(())`.
    ///
    /// # Arguments
    /// * `_txs` - A vector of transaction hashes to be deleted (ignored).
    ///
    fn delete_all(&self, _txs: Vec<B256>) -> Result<(), BlobStoreError> {
        Ok(())
    }

    /// Performs cleanup operations and returns statistics about the cleanup process.
    ///
    /// This method does nothing and always returns a default `BlobStoreCleanupStat` instance.
    ///
    fn cleanup(&self) -> BlobStoreCleanupStat {
        BlobStoreCleanupStat::default()
    }

    /// Retrieves the decoded blob data for the given transaction hash.
    ///
    /// This method does nothing and always returns `Ok(None)`.
    ///
    /// # Arguments
    /// * `_tx` - The transaction hash to retrieve the blob for (ignored).
    ///
    fn get(&self, _tx: B256) -> Result<Option<BlobTransactionSidecar>, BlobStoreError> {
        Ok(None)
    }

    /// Checks if the given transaction hash is present in the blob store.
    ///
    /// This method does nothing and always returns `Ok(false)`.
    ///
    /// # Arguments
    /// * `_tx` - The transaction hash to check (ignored).
    ///
    fn contains(&self, _tx: B256) -> Result<bool, BlobStoreError> {
        Ok(false)
    }

    /// Retrieves all decoded blob data for the given transaction hashes.
    ///
    /// This method does nothing and always returns `Ok(vec![])`.
    ///
    /// # Arguments
    /// * `_txs` - A vector of transaction hashes to retrieve the blobs for (ignored).
    ///
    fn get_all(
        &self,
        _txs: Vec<B256>,
    ) -> Result<Vec<(B256, BlobTransactionSidecar)>, BlobStoreError> {
        Ok(vec![])
    }

    /// Retrieves the exact `BlobTransactionSidecar` for the given transaction hashes in the exact order they were requested.
    ///
    /// This method returns an error if any transaction hashes are provided, otherwise it returns `Ok(vec![])`.
    ///
    /// # Arguments
    /// * `txs` - A vector of transaction hashes to retrieve the blobs for.
    ///
    /// # Errors
    /// Returns `BlobStoreError::MissingSidecar` if any transaction hashes are provided.
    ////
    fn get_exact(&self, txs: Vec<B256>) -> Result<Vec<BlobTransactionSidecar>, BlobStoreError> {
        if txs.is_empty() {
            return Ok(vec![])
        }
        Err(BlobStoreError::MissingSidecar(txs[0]))
    }

    /// Provides a hint of the total size of the blob data in the store.
    ///
    /// This method does nothing and always returns `Some(0)`.
    ///
    fn data_size_hint(&self) -> Option<usize> {
        Some(0)
    }

    /// Returns the number of blobs in the store.
    ///
    /// This method does nothing and always returns `0`.
    ///
    fn blobs_len(&self) -> usize {
        0
    }
}
