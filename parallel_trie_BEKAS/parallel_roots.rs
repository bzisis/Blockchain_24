// Import necessary crates and modules
use crate::{stats::ParallelTrieTracker, storage_root_targets::StorageRootTargets};
use alloy_rlp::{BufMut, Encodable};
use rayon::prelude::*;
use reth_db_api::database::Database;
use reth_execution_errors::StorageRootError;
use reth_primitives::B256;
use reth_provider::{providers::ConsistentDbView, DatabaseProviderFactory, ProviderError};
use reth_trie::{
    hashed_cursor::{HashedCursorFactory, HashedPostStateCursorFactory},
    node_iter::{TrieElement, TrieNodeIter},
    trie_cursor::TrieCursorFactory,
    updates::TrieUpdates,
    walker::TrieWalker,
    HashBuilder, HashedPostState, Nibbles, StorageRoot, TrieAccount,
};
use std::collections::HashMap;
use thiserror::Error;
use tracing::*;

// Optionally include metrics if feature is enabled
#[cfg(feature = "metrics")]
use crate::metrics::ParallelStateRootMetrics;

/// Parallel incremental state root calculator.
///
/// The calculator starts off by pre-computing storage roots of changed
/// accounts in parallel. Once that's done, it proceeds to walking the state
/// trie retrieving the pre-computed storage roots when needed.
///
/// Internally, the calculator uses [`ConsistentDbView`] since
/// it needs to rely on database state saying the same until
/// the last transaction is open.
/// See docs of using [`ConsistentDbView`] for caveats.
///
/// If possible, use more optimized `AsyncStateRoot` instead.
#[derive(Debug)]
pub struct ParallelStateRoot<DB, Provider> {
    /// Consistent view of the database.
    view: ConsistentDbView<DB, Provider>,
    /// Changed hashed state.
    hashed_state: HashedPostState,
    /// Parallel state root metrics.
    #[cfg(feature = "metrics")]
    metrics: ParallelStateRootMetrics,
}

impl<DB, Provider> ParallelStateRoot<DB, Provider> {
    /// Create new parallel state root calculator.
    pub fn new(view: ConsistentDbView<DB, Provider>, hashed_state: HashedPostState) -> Self {
        Self {
            view,
            hashed_state,
            #[cfg(feature = "metrics")]
            metrics: ParallelStateRootMetrics::default(),
        }
    }
}

impl<DB, Provider> ParallelStateRoot<DB, Provider>
where
    DB: Database,
    Provider: DatabaseProviderFactory<DB> + Send + Sync,
{
    /// Calculate incremental state root in parallel.
    pub fn incremental_root(self) -> Result<B256, ParallelStateRootError> {
        self.calculate(false).map(|(root, _)| root)
    }

    /// Calculate incremental state root with updates in parallel.
    pub fn incremental_root_with_updates(
        self,
    ) -> Result<(B256, TrieUpdates), ParallelStateRootError> {
        self.calculate(true)
    }

    /// Internal method to perform state root calculation.
    fn calculate(
        self,
        retain_updates: bool,
    ) -> Result<(B256, TrieUpdates), ParallelStateRootError> {
        // Initialize a tracker to collect statistics during computation
        let mut tracker = ParallelTrieTracker::default();
        // Construct prefix sets from the hashed state
        let prefix_sets = self.hashed_state.construct_prefix_sets().freeze();
        // Create storage root targets for changed accounts
        let storage_root_targets = StorageRootTargets::new(
            self.hashed_state.accounts.keys().copied(),
            prefix_sets.storage_prefix_sets,
        );
        // Convert hashed state to a sorted form
        let hashed_state_sorted = self.hashed_state.into_sorted();

        // Set the number of precomputed storage roots in the tracker
        tracker.set_precomputed_storage_roots(storage_root_targets.len() as u64);
        debug!(target: "trie::parallel_state_root", len = storage_root_targets.len(), "pre-calculating storage roots");

        // Perform parallel computation of storage roots for changed accounts
        let mut storage_roots = storage_root_targets
            .into_par_iter()
            .map(|(hashed_address, prefix_set)| {
                let provider_ro = self.view.provider_ro()?;
                // Calculate storage root for each account
                let storage_root_result = StorageRoot::new_hashed(
                    provider_ro.tx_ref(),
                    HashedPostStateCursorFactory::new(provider_ro.tx_ref(), &hashed_state_sorted),
                    hashed_address,
                    #[cfg(feature = "metrics")]
                    self.metrics.storage_trie.clone(),
                )
                .with_prefix_set(prefix_set)
                .calculate(retain_updates);
                Ok((hashed_address, storage_root_result?))
            })
            .collect::<Result<HashMap<_, _>, ParallelStateRootError>>()?;

        // Trace message for starting to calculate state root
        trace!(target: "trie::parallel_state_root", "calculating state root");
        let mut trie_updates = TrieUpdates::default();

        // Obtain read-only provider for database operations
        let provider_ro = self.view.provider_ro()?;
        // Create hashed cursor factory for the sorted hashed state
        let hashed_cursor_factory =
            HashedPostStateCursorFactory::new(provider_ro.tx_ref(), &hashed_state_sorted);
        // Obtain trie cursor factory for database operations
        let trie_cursor_factory = provider_ro.tx_ref();

        // Create a trie walker with account trie cursor and account prefix set
        let walker = TrieWalker::new(
            trie_cursor_factory.account_trie_cursor().map_err(ProviderError::Database)?,
            prefix_sets.account_prefix_set,
        )
        .with_deletions_retained(retain_updates);
        // Create an iterator over trie nodes using walker and hashed account cursor
        let mut account_node_iter = TrieNodeIter::new(
            walker,
            hashed_cursor_factory.hashed_account_cursor().map_err(ProviderError::Database)?,
        );

        // Initialize hash builder to construct trie root
        let mut hash_builder = HashBuilder::default().with_updates(retain_updates);
        // Buffer for encoding account RLP
        let mut account_rlp = Vec::with_capacity(128);

        // Iterate over each trie node
        while let Some(node) = account_node_iter.try_next().map_err(ProviderError::Database)? {
            match node {
                // Handle branch nodes
                TrieElement::Branch(node) => {
                    hash_builder.add_branch(node.key, node.value, node.children_are_in_trie);
                }
                // Handle leaf nodes (accounts)
                TrieElement::Leaf(hashed_address, account) => {
                    // Retrieve precomputed storage root for the account
                    let (storage_root, _, updates) = match storage_roots.remove(&hashed_address) {
                        Some(result) => result,
                        // If storage root is not precomputed, calculate it
                        None => {
                            tracker.inc_missed_leaves();
                            StorageRoot::new_hashed(
                                trie_cursor_factory,
                                hashed_cursor_factory.clone(),
                                hashed_address,
                                #[cfg(feature = "metrics")]
                                self.metrics.storage_trie.clone(),
                            )
                            .calculate(retain_updates)?
                        }
                    };

                    // Insert storage updates if retaining updates
                    if retain_updates {
                        trie_updates.insert_storage_updates(hashed_address, updates);
                    }

                    // Encode account with its storage root into RLP format
                    account_rlp.clear();
                    let account = TrieAccount::from((account, storage_root));
                    account.encode(&mut account_rlp as &mut dyn BufMut);
                    // Add the encoded leaf to hash builder
                    hash_builder.add_leaf(Nibbles::unpack(hashed_address), &account_rlp);
                }
            }
        }

        // Compute the final root hash of the trie
        let root = hash_builder.root();

        // Finalize trie updates
        trie_updates.finalize(
            account_node_iter.walker,
            hash_builder,
            prefix_sets.destroyed_accounts,
        );

        // Finish tracking statistics
        let stats = tracker.finish();

        // Record metrics if feature is enabled
        #[cfg(feature = "metrics")]
        self.metrics.record_state_trie(stats);

        // Trace message for completing state root calculation
        trace!(
            target: "trie::parallel_state_root",
            %root,
            duration = ?stats.duration(),
            branches_added = stats.branches_added(),
            leaves_added = stats.leaves_added(),
            missed_leaves = stats.missed_leaves(),
            precomputed_storage_roots = stats.precomputed_storage_roots(),
            "calculated state root"
        );

        // Return computed root hash and trie updates
        Ok((root, trie_updates))
    }
}

/// Error during parallel state root calculation.
#[derive(Error, Debug)]
pub enum ParallelStateRootError {
    /// Error while calculating storage root.
    #[error(transparent)]
    StorageRoot(#[from] StorageRootError),
    /// Provider error.
    #[error(transparent)]
    Provider(#[from] ProviderError),
}

impl From<ParallelStateRootError> for ProviderError {
    fn from(error: ParallelStateRootError) -> Self {
        match error {
            ParallelStateRootError::Provider(error) => error,
            ParallelStateRootError::StorageRoot(StorageRootError::DB(error)) => {
                Self::Database(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use reth_primitives::{keccak256, Account, Address, StorageEntry, U256};
    use reth_provider::{test_utils::create_test_provider_factory, HashingWriter};
    use reth_trie::{test_utils, HashedStorage};

    #[tokio::test]
    async fn random_parallel_root() {
        // Create a test provider factory
        let factory = create_test_provider_factory();
        // Create a consistent database view
        let consistent_view = ConsistentDbView::new(factory.clone(), None);

        let mut rng = rand::thread_rng();
        let mut state = (0..100)
            .map(|_| {
                // Generate random address and account
                let address = Address::random();
                let account =
                    Account { balance: U256::from(rng.gen::<u64>()), ..Default::default() };
                let mut storage = HashMap::<B256, U256>::default();
                let has_storage = rng.gen_bool(0.7);
                if has_storage {
                    // Populate storage with random entries
                    for _ in 0..100 {
                        storage.insert(
                            B256::from(U256::from(rng.gen::<u64>())),
                            U256::from(rng.gen::<u64>()),
                        );
                    }
                }
                (address, (account, storage))
            })
            .collect::<HashMap<_, _>>();

        {
            // Obtain read-write provider and insert account data for hashing
            let provider_rw = factory.provider_rw().unwrap();
            provider_rw
                .insert_account_for_hashing(
                    state.iter().map(|(address, (account, _))| (*address, Some(*account))),
                )
                .unwrap();
            // Insert storage data for hashing
            provider_rw
                .insert_storage_for_hashing(state.iter().map(|(address, (_, storage))| {
                    (
                        *address,
                        storage
                            .iter()
                            .map(|(slot, value)| StorageEntry { key: *slot, value: *value }),
                    )
                }))
                .unwrap();
        }

        // Create hashed state from the initial state
        let hashed_state = test_utils::hash_state(state);
        // Create a parallel state root calculator
        let parallel_state_root = ParallelStateRoot::new(consistent_view, hashed_state);
        // Compute the incremental root
        let root = parallel_state_root.incremental_root().unwrap();
        // Validate the root with expected value
        assert_eq!(
            root,
            B256::from_slice(
                &hex::decode(
                    "5c6b5a4f2334d2f3b1e9e2bb0c6b5c229d5f2b1c0f8b5b3c5e6f3e2b5b4f5c6b"
                )
                .unwrap()
            )
        );
    }
}
