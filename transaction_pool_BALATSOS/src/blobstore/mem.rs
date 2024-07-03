// Author: Zisis Balatsos

use crate::blobstore::{
    BlobStore, BlobStoreCleanupStat, BlobStoreError, BlobStoreSize, BlobTransactionSidecar,
};
use parking_lot::RwLock;
use reth_primitives::B256;
use std::{collections::HashMap, sync::Arc};

/// An in-memory blob store.
///
/// This struct uses an inner struct wrapped in an `Arc` for shared ownership and a `RwLock`
/// for concurrent read/write access to the underlying storage.
///

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InMemoryBlobStore {
    inner: Arc<InMemoryBlobStoreInner>,
}

/// Inner struct for `InMemoryBlobStore` containing the actual storage and size tracker.

#[derive(Debug, Default)]
struct InMemoryBlobStoreInner {
    /// Storage for all blob data.
    /// A hash map that stores the blob data with the transaction hash as the key.
    store: RwLock<HashMap<B256, BlobTransactionSidecar>>,
    /// Tracks the size and number of blobs in the store.
    size_tracker: BlobStoreSize,
}

/// Implementation of the `PartialEq` trait for `InMemoryBlobStoreInner`.
/// This allows comparing two `InMemoryBlobStoreInner` instances for equality.
impl PartialEq for InMemoryBlobStoreInner {
    fn eq(&self, other: &Self) -> bool {
        self.store.read().eq(&other.store.read())
    }
}

/// Implementation of the `BlobStore` trait for `InMemoryBlobStore`.
impl BlobStore for InMemoryBlobStore {
    /// Inserts a single blob into the store.
    ///
    /// # Arguments
    /// * `tx` - The transaction hash.
    /// * `data` - The blob data to be stored.
    fn insert(&self, tx: B256, data: BlobTransactionSidecar) -> Result<(), BlobStoreError> {
        let mut store = self.inner.store.write();
        self.inner.size_tracker.add_size(insert_size(&mut store, tx, data));
        self.inner.size_tracker.update_len(store.len());
        Ok(())
    }


    /// Inserts multiple blobs into the store.
    ///
    /// # Arguments
    /// * `txs` - A vector of tuples containing the transaction hash and corresponding blob data.
    fn insert_all(&self, txs: Vec<(B256, BlobTransactionSidecar)>) -> Result<(), BlobStoreError> {
        if txs.is_empty() {
            return Ok(())
        }
        let mut store = self.inner.store.write();
        let mut total_add = 0;
        for (tx, data) in txs {
            let add = insert_size(&mut store, tx, data);
            total_add += add;
        }
        self.inner.size_tracker.add_size(total_add);
        self.inner.size_tracker.update_len(store.len());
        Ok(())
    }

    /// Deletes a single blob from the store.
    ///
    /// # Arguments
    /// * `tx` - The transaction hash to be deleted.
    fn delete(&self, tx: B256) -> Result<(), BlobStoreError> {
        let mut store = self.inner.store.write();
        let sub = remove_size(&mut store, &tx);
        self.inner.size_tracker.sub_size(sub);
        self.inner.size_tracker.update_len(store.len());
        Ok(())
    }

    /// Deletes multiple blobs from the store.
    ///
    /// # Arguments
    /// * `txs` - A vector of transaction hashes to be deleted.
    fn delete_all(&self, txs: Vec<B256>) -> Result<(), BlobStoreError> {
        if txs.is_empty() {
            return Ok(())
        }
        let mut store = self.inner.store.write();
        let mut total_sub = 0;
        for tx in txs {
            total_sub += remove_size(&mut store, &tx);
        }
        self.inner.size_tracker.sub_size(total_sub);
        self.inner.size_tracker.update_len(store.len());
        Ok(())
    }

    /// Performs cleanup operations. Currently, this is a no-op for the in-memory store.
    fn cleanup(&self) -> BlobStoreCleanupStat {
        BlobStoreCleanupStat::default()
    }

    // Retrieves the decoded blob data for the given transaction hash.
    ///
    /// # Arguments
    /// * `tx` - The transaction hash to retrieve the blob for.
    ///
    fn get(&self, tx: B256) -> Result<Option<BlobTransactionSidecar>, BlobStoreError> {
        let store = self.inner.store.read();
        Ok(store.get(&tx).cloned())
    }

    /// Checks if the store contains the blob for the given transaction hash.
    ///
    /// # Arguments
    /// * `tx` - The transaction hash to check.
    ///
    fn contains(&self, tx: B256) -> Result<bool, BlobStoreError> {
        let store = self.inner.store.read();
        Ok(store.contains_key(&tx))
    }

    /// Retrieves multiple blobs for the given transaction hashes.
    ///
    /// # Arguments
    /// * `txs` - A vector of transaction hashes to retrieve the blobs for.
    ///
    fn get_all(
        &self,
        txs: Vec<B256>,
    ) -> Result<Vec<(B256, BlobTransactionSidecar)>, BlobStoreError> {
        let mut items = Vec::with_capacity(txs.len());
        let store = self.inner.store.read();
        for tx in txs {
            if let Some(item) = store.get(&tx) {
                items.push((tx, item.clone()));
            }
        }

        Ok(items)
    }

    /// Retrieves exactly the blobs corresponding to the given transaction hashes.
    ///
    /// # Arguments
    /// * `txs` - A vector of transaction hashes to retrieve the blobs for.
    ///
    fn get_exact(&self, txs: Vec<B256>) -> Result<Vec<BlobTransactionSidecar>, BlobStoreError> {
        let mut items = Vec::with_capacity(txs.len());
        let store = self.inner.store.read();
        for tx in txs {
            if let Some(item) = store.get(&tx) {
                items.push(item.clone());
            } else {
                return Err(BlobStoreError::MissingSidecar(tx))
            }
        }

        Ok(items)
    }

    /// Provides a hint of the total size of the blob data in the store.
    fn data_size_hint(&self) -> Option<usize> {
        Some(self.inner.size_tracker.data_size())
    }

    /// Returns the number of blobs in the store.
    fn blobs_len(&self) -> usize {
        self.inner.size_tracker.blobs_len()
    }
}

/// Removes the given blob from the store and returns the size of the blob that was removed.
///
/// # Arguments
/// * `store` - The hash map storing the blobs.
/// * `tx` - The transaction hash of the blob to be removed.
///

#[inline]
fn remove_size(store: &mut HashMap<B256, BlobTransactionSidecar>, tx: &B256) -> usize {
    store.remove(tx).map(|rem| rem.size()).unwrap_or_default()
}

/// Inserts the given blob into the store and returns the size of the blob that was added.
///
/// We don't need to handle the size updates for replacements because transactions are unique.

/// Inserts the given blob into the store and returns the size of the blob that was added.
///
/// We don't need to handle the size updates for replacements because transactions are unique.
///
/// # Arguments
/// * `store` - The hash map storing the blobs.
/// * `tx` - The transaction hash of the blob to be inserted.
/// * `blob` - The blob data to be inserted.
///

#[inline]
fn insert_size(
    store: &mut HashMap<B256, BlobTransactionSidecar>,
    tx: B256,
    blob: BlobTransactionSidecar,
) -> usize {
    let add = blob.size();
    store.insert(tx, blob);
    add
}
