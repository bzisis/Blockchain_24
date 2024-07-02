use reth_blockchain_tree_api::{
    self,
    error::{BlockchainTreeError, CanonicalError, InsertBlockError, ProviderError},
    BlockValidationKind, BlockchainTreeEngine, BlockchainTreeViewer, CanonicalOutcome,
    InsertPayloadOk,
};
use reth_primitives::{
    BlockHash, BlockNumHash, BlockNumber, Receipt, SealedBlock, SealedBlockWithSenders,
    SealedHeader,
};
use reth_provider::{
    BlockchainTreePendingStateProvider, CanonStateNotificationSender, CanonStateNotifications,
    CanonStateSubscriptions, FullExecutionDataProvider,
};
use reth_storage_errors::provider::ProviderResult;
use std::collections::BTreeMap;

/// A `BlockchainTree` that does nothing.
///
/// Caution: this is only intended for testing purposes, or for wiring components together.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct NoopBlockchainTree {
    /// Broadcast channel for canon state changes notifications.
    pub canon_state_notification_sender: Option<CanonStateNotificationSender>,
}

impl NoopBlockchainTree {
    /// Create a new `NoopBlockchainTree` with a canon state notification sender.
    pub const fn with_canon_state_notifications(
        canon_state_notification_sender: CanonStateNotificationSender,
    ) -> Self {
        Self { canon_state_notification_sender: Some(canon_state_notification_sender) }
    }
}

impl BlockchainTreeEngine for NoopBlockchainTree {
    fn buffer_block(&self, _block: SealedBlockWithSenders) -> Result<(), InsertBlockError> {
        Ok(())
    }

    fn insert_block(
        &self,
        block: SealedBlockWithSenders,
        _validation_kind: BlockValidationKind,
    ) -> Result<InsertPayloadOk, InsertBlockError> {
        Err(InsertBlockError::tree_error(
            BlockchainTreeError::BlockHashNotFoundInChain { block_hash: block.hash() },
            block.block,
        ))
    }

    fn finalize_block(&self, _finalized_block: BlockNumber) -> ProviderResult<()> {
        Ok(())
    }

    fn connect_buffered_blocks_to_canonical_hashes_and_finalize(
        &self,
        _last_finalized_block: BlockNumber,
    ) -> Result<(), CanonicalError> {
        Ok(())
    }

    fn connect_buffered_blocks_to_canonical_hashes(&self) -> Result<(), CanonicalError> {
        Ok(())
    }

    fn make_canonical(&self, block_hash: BlockHash) -> Result<CanonicalOutcome, CanonicalError> {
        Err(BlockchainTreeError::BlockHashNotFoundInChain { block_hash }.into())
    }

    fn update_block_hashes_and_clear_buffered(
        &self,
    ) -> Result<BTreeMap<BlockNumber, BlockHash>, CanonicalError> {
        Ok(BTreeMap::new())
    }
}

impl BlockchainTreeViewer for NoopBlockchainTree {
    fn header_by_hash(&self, _hash: BlockHash) -> Option<SealedHeader> {
        None
    }

    fn block_by_hash(&self, _hash: BlockHash) -> Option<SealedBlock> {
        None
    }

    fn block_with_senders_by_hash(&self, _hash: BlockHash) -> Option<SealedBlockWithSenders> {
        None
    }

    fn buffered_header_by_hash(&self, _block_hash: BlockHash) -> Option<SealedHeader> {
        None
    }

    fn is_canonical(&self, _block_hash: BlockHash) -> Result<bool, ProviderError> {
        Ok(false)
    }

    fn lowest_buffered_ancestor(&self, _hash: BlockHash) -> Option<SealedBlockWithSenders> {
        None
    }

    fn canonical_tip(&self) -> BlockNumHash {
        Default::default()
    }

    fn pending_block_num_hash(&self) -> Option<BlockNumHash> {
        None
    }

    fn pending_block_and_receipts(&self) -> Option<(SealedBlock, Vec<Receipt>)> {
        None
    }

    fn receipts_by_block_hash(&self, _block_hash: BlockHash) -> Option<Vec<Receipt>> {
        None
    }
}

impl BlockchainTreePendingStateProvider for NoopBlockchainTree {
    fn find_pending_state_provider(
        &self,
        _block_hash: BlockHash,
    ) -> Option<Box<dyn FullExecutionDataProvider>> {
        None
    }
}

impl CanonStateSubscriptions for NoopBlockchainTree {
    fn subscribe_to_canonical_state(&self) -> CanonStateNotifications {
        self.canon_state_notification_sender
            .as_ref()
            .map(|sender| sender.subscribe())
            .unwrap_or_else(|| CanonStateNotificationSender::new(1).subscribe())
    }
}use reth_blockchain_tree_api::{
    self,
    error::{BlockchainTreeError, CanonicalError, InsertBlockError, ProviderError},
    BlockValidationKind, BlockchainTreeEngine, BlockchainTreeViewer, CanonicalOutcome,
    InsertPayloadOk,
};
use reth_primitives::{
    BlockHash, BlockNumHash, BlockNumber, Receipt, SealedBlock, SealedBlockWithSenders,
    SealedHeader,
};
use reth_provider::{
    BlockchainTreePendingStateProvider, CanonStateNotificationSender, CanonStateNotifications,
    CanonStateSubscriptions, FullExecutionDataProvider,
};
use reth_storage_errors::provider::ProviderResult;
use std::collections::BTreeMap;

/// A `BlockchainTree` that does nothing.
///
/// Caution: this is only intended for testing purposes, or for wiring components together.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct NoopBlockchainTree {
    /// Broadcast channel for canon state changes notifications.
    pub canon_state_notification_sender: Option<CanonStateNotificationSender>,
}

impl NoopBlockchainTree {
    /// Create a new `NoopBlockchainTree` with a canon state notification sender.
    pub const fn with_canon_state_notifications(
        canon_state_notification_sender: CanonStateNotificationSender,
    ) -> Self {
        Self { canon_state_notification_sender: Some(canon_state_notification_sender) }
    }
}

impl BlockchainTreeEngine for NoopBlockchainTree {
    /// Buffers a block in the blockchain tree.
    ///
    /// In this no-op implementation, this function does nothing and always returns `Ok`.
    fn buffer_block(&self, _block: SealedBlockWithSenders) -> Result<(), InsertBlockError> {
        Ok(())
    }

    /// Inserts a block into the blockchain tree.
    ///
    /// In this no-op implementation, this function always returns an error indicating that the block hash was not found in the chain.
    fn insert_block(
        &self,
        block: SealedBlockWithSenders,
        _validation_kind: BlockValidationKind,
    ) -> Result<InsertPayloadOk, InsertBlockError> {
        Err(InsertBlockError::tree_error(
            BlockchainTreeError::BlockHashNotFoundInChain { block_hash: block.hash() },
            block.block,
        ))
    }

    /// Finalizes a block in the blockchain tree.
    ///
    /// In this no-op implementation, this function does nothing and always returns `Ok`.
    fn finalize_block(&self, _finalized_block: BlockNumber) -> ProviderResult<()> {
        Ok(())
    }

    /// Connects buffered blocks to canonical hashes and finalizes them.
    ///
    /// In this no-op implementation, this function does nothing and always returns `Ok`.
    fn connect_buffered_blocks_to_canonical_hashes_and_finalize(
        &self,
        _last_finalized_block: BlockNumber,
    ) -> Result<(), CanonicalError> {
        Ok(())
    }

    /// Connects buffered blocks to canonical hashes.
    ///
    /// In this no-op implementation, this function does nothing and always returns `Ok`.
    fn connect_buffered_blocks_to_canonical_hashes(&self) -> Result<(), CanonicalError> {
        Ok(())
    }

    /// Makes a block canonical in the blockchain tree.
    ///
    /// In this no-op implementation, this function always returns an error indicating that the block hash was not found in the chain.
    fn make_canonical(&self, block_hash: BlockHash) -> Result<CanonicalOutcome, CanonicalError> {
        Err(BlockchainTreeError::BlockHashNotFoundInChain { block_hash }.into())
    }

    /// Updates block hashes and clears buffered blocks.
    ///
    /// In this no-op implementation, this function does nothing and always returns an empty map.
    fn update_block_hashes_and_clear_buffered(
        &self,
    ) -> Result<BTreeMap<BlockNumber, BlockHash>, CanonicalError> {
        Ok(BTreeMap::new())
    }
}

impl BlockchainTreeViewer for NoopBlockchainTree {
    /// Gets the header of a block by its hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn header_by_hash(&self, _hash: BlockHash) -> Option<SealedHeader> {
        None
    }

    /// Gets a block by its hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn block_by_hash(&self, _hash: BlockHash) -> Option<SealedBlock> {
        None
    }

    /// Gets a block with senders by its hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn block_with_senders_by_hash(&self, _hash: BlockHash) -> Option<SealedBlockWithSenders> {
        None
    }

    /// Gets a buffered header by its hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn buffered_header_by_hash(&self, _block_hash: BlockHash) -> Option<SealedHeader> {
        None
    }

    /// Checks if a block is canonical.
    ///
    /// In this no-op implementation, this function always returns `false`.
    fn is_canonical(&self, _block_hash: BlockHash) -> Result<bool, ProviderError> {
        Ok(false)
    }

    /// Gets the lowest buffered ancestor of a block by its hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn lowest_buffered_ancestor(&self, _hash: BlockHash) -> Option<SealedBlockWithSenders> {
        None
    }

    /// Gets the canonical tip of the blockchain.
    ///
    /// In this no-op implementation, this function always returns a default `BlockNumHash`.
    fn canonical_tip(&self) -> BlockNumHash {
        Default::default()
    }

    /// Gets the pending block number and hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn pending_block_num_hash(&self) -> Option<BlockNumHash> {
        None
    }

    /// Gets the pending block and its receipts.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn pending_block_and_receipts(&self) -> Option<(SealedBlock, Vec<Receipt>)> {
        None
    }

    /// Gets receipts by block hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn receipts_by_block_hash(&self, _block_hash: BlockHash) -> Option<Vec<Receipt>> {
        None
    }
}

impl BlockchainTreePendingStateProvider for NoopBlockchainTree {
    /// Finds the pending state provider for a block by its hash.
    ///
    /// In this no-op implementation, this function always returns `None`.
    fn find_pending_state_provider(
        &self,
        _block_hash: BlockHash,
    ) -> Option<Box<dyn FullExecutionDataProvider>> {
        None
    }
}

impl CanonStateSubscriptions for NoopBlockchainTree {
    /// Subscribes to canonical state changes.
    ///
    /// In this no-op implementation, this function returns a default subscription if no notification sender is set.
    fn subscribe_to_canonical_state(&self) -> CanonStateNotifications {
        self.canon_state_notification_sender
            .as_ref()
            .map(|sender| sender.subscribe())
            .unwrap_or_else(|| CanonStateNotificationSender::new(1).subscribe())
    }
}
