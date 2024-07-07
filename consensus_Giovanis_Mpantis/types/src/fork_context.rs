use parking_lot::RwLock;

use crate::{ChainSpec, EthSpec, ForkName, Hash256, Slot};
use std::collections::HashMap;

/// Provides fork-specific information such as the current fork name and the fork digests corresponding to every valid fork.
///
/// This struct manages information about the current fork, mappings between fork names and their corresponding digest,
/// and provides methods to query and update this information.
#[derive(Debug)]
pub struct ForkContext {
    /// The current fork name, protected by a reader-writer lock for concurrent access.
    current_fork: RwLock<ForkName>,
    /// Maps each fork name to its corresponding digest (context bytes).
    fork_to_digest: HashMap<ForkName, [u8; 4]>,
    /// Maps each digest (context bytes) to its corresponding fork name.
    digest_to_fork: HashMap<[u8; 4], ForkName>,
    /// The chain specification associated with this `ForkContext`.
    pub spec: ChainSpec,
}

impl ForkContext {
    /// Creates a new `ForkContext` object by enumerating all enabled forks and computing their fork digest.
    ///
    /// A fork is considered enabled if its activation slot in the `ChainSpec` is defined (not `None`).
    ///
    /// # Arguments
    ///
    /// * `current_slot` - The current slot in the chain, used to determine the initial current fork.
    /// * `genesis_validators_root` - The root hash of the genesis validators, used for computing fork digests.
    /// * `spec` - The chain specification containing fork configuration.
    ///
    /// # Returns
    ///
    /// A `ForkContext` object initialized with the computed fork digests and current fork.
    pub fn new<E: EthSpec>(
        current_slot: Slot,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> Self {
        let mut fork_to_digest = vec![(
            ForkName::Base,
            ChainSpec::compute_fork_digest(spec.genesis_fork_version, genesis_validators_root),
        )];

        if let Some(altair_fork_epoch) = spec.altair_fork_epoch {
            if current_slot >= altair_fork_epoch.start_slot(spec) {
                fork_to_digest.push((
                    ForkName::Altair,
                    ChainSpec::compute_fork_digest(spec.altair_fork_version, genesis_validators_root),
                ));
            }
        }

        if let Some(bellatrix_fork_epoch) = spec.bellatrix_fork_epoch {
            if current_slot >= bellatrix_fork_epoch.start_slot(spec) {
                fork_to_digest.push((
                    ForkName::Bellatrix,
                    ChainSpec::compute_fork_digest(spec.bellatrix_fork_version, genesis_validators_root),
                ));
            }
        }

        // Add more forks as needed (e.g., Capella, Deneb, Electra) based on their activation epochs.

        let fork_to_digest: HashMap<ForkName, [u8; 4]> = fork_to_digest.into_iter().collect();

        let digest_to_fork = fork_to_digest
            .clone()
            .into_iter()
            .map(|(k, v)| (v, k))
            .collect();

        Self {
            current_fork: RwLock::new(spec.fork_name_at_slot::<E>(current_slot)),
            fork_to_digest,
            digest_to_fork,
            spec: spec.clone(),
        }
    }

    /// Returns `true` if the provided `fork_name` exists in the `ForkContext` object.
    pub fn fork_exists(&self, fork_name: ForkName) -> bool {
        self.fork_to_digest.contains_key(&fork_name)
    }

    /// Returns the current fork name.
    pub fn current_fork(&self) -> ForkName {
        *self.current_fork.read()
    }

    /// Updates the `current_fork` field to a new fork name.
    pub fn update_current_fork(&self, new_fork: ForkName) {
        *self.current_fork.write() = new_fork;
    }

    /// Returns the context bytes (fork digest) corresponding to the genesis fork version.
    ///
    /// # Returns
    ///
    /// The context bytes (fork digest) of the genesis fork version.
    ///
    /// # Panics
    ///
    /// Panics if the `ForkContext` does not contain the genesis context bytes.
    pub fn genesis_context_bytes(&self) -> [u8; 4] {
        *self
            .fork_to_digest
            .get(&ForkName::Base)
            .expect("ForkContext must contain genesis context bytes")
    }

    /// Returns the fork name associated with the given context bytes (fork digest).
    ///
    /// # Arguments
    ///
    /// * `context` - The context bytes (fork digest) to look up.
    ///
    /// # Returns
    ///
    /// The fork name corresponding to the provided context bytes, or `None` if no matching fork is found.
    pub fn from_context_bytes(&self, context: [u8; 4]) -> Option<&ForkName> {
        self.digest_to_fork.get(&context)
    }

    /// Returns the context bytes (fork digest) corresponding to a given fork name.
    ///
    /// # Arguments
    ///
    /// * `fork_name` - The fork name to retrieve the context bytes for.
    ///
    /// # Returns
    ///
    /// The context bytes (fork digest) corresponding to the provided fork name, or `None` if the fork name has not been initialized.
    pub fn to_context_bytes(&self, fork_name: ForkName) -> Option<[u8; 4]> {
        self.fork_to_digest.get(&fork_name).cloned()
    }

    /// Returns a vector of all fork digests currently stored in the `ForkContext`.
    ///
    /// # Returns
    ///
    /// A vector containing all fork digests stored in the `ForkContext`.
    pub fn all_fork_digests(&self) -> Vec<[u8; 4]> {
        self.digest_to_fork.keys().cloned().collect()
    }
}
