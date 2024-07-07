use crate::{ActivationQueue, BeaconStateError, ChainSpec, Epoch, Hash256, Slot};
use safe_arith::{ArithError, SafeArith};
use std::sync::Arc;

/// Cache of values which are uniquely determined at the start of an epoch.
///
/// The values are fixed with respect to the last block of the _prior_ epoch, which we refer
/// to as the "decision block". This cache is very similar to the `BeaconProposerCache` in that
/// beacon proposers are determined at exactly the same time as the values in this cache, so
/// the keys for the two caches are identical.
#[derive(Debug, PartialEq, Eq, Clone, Default, arbitrary::Arbitrary)]
pub struct EpochCache {
    inner: Option<Arc<Inner>>,
}

/// Inner structure of `EpochCache`, containing the cached values for an epoch.
#[derive(Debug, PartialEq, Eq, Clone, arbitrary::Arbitrary)]
struct Inner {
    /// Unique identifier for this cache, which can be used to check its validity before use
    /// with any `BeaconState`.
    key: EpochCacheKey,
    /// Effective balance for every validator in this epoch.
    effective_balances: Vec<u64>,
    /// Base rewards for every effective balance increment (currently 0..32 ETH).
    ///
    /// Keyed by `effective_balance / effective_balance_increment`.
    base_rewards: Vec<u64>,
    /// Validator activation queue.
    activation_queue: ActivationQueue,
    /// Effective balance increment.
    effective_balance_increment: u64,
}

/// Key used to uniquely identify an `EpochCache`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, arbitrary::Arbitrary)]
pub struct EpochCacheKey {
    /// The epoch number.
    pub epoch: Epoch,
    /// The root hash of the decision block for the epoch.
    pub decision_block_root: Hash256,
}

/// Errors that can occur when interacting with `EpochCache`.
#[derive(Debug, PartialEq, Clone)]
pub enum EpochCacheError {
    /// The cached epoch does not match the current epoch.
    IncorrectEpoch { cache: Epoch, state: Epoch },
    /// The decision block root in the cache does not match the state's decision block root.
    IncorrectDecisionBlock { cache: Hash256, state: Hash256 },
    /// Validator index is out of bounds for the cached values.
    ValidatorIndexOutOfBounds { validator_index: usize },
    /// Effective balance index is out of bounds for the cached values.
    EffectiveBalanceOutOfBounds { effective_balance_eth: usize },
    /// Invalid slot encountered.
    InvalidSlot { slot: Slot },
    /// Arithmetic error occurred.
    Arith(ArithError),
    /// Beacon state error occurred.
    BeaconState(BeaconStateError),
    /// The `EpochCache` has not been initialized.
    CacheNotInitialized,
}

impl From<BeaconStateError> for EpochCacheError {
    fn from(e: BeaconStateError) -> Self {
        Self::BeaconState(e)
    }
}

impl From<ArithError> for EpochCacheError {
    fn from(e: ArithError) -> Self {
        Self::Arith(e)
    }
}

impl EpochCache {
    /// Creates a new `EpochCache` instance.
    ///
    /// # Parameters
    ///
    /// - `key`: The key identifying this cache.
    /// - `effective_balances`: Effective balances for every validator in this epoch.
    /// - `base_rewards`: Base rewards for effective balances increments.
    /// - `activation_queue`: Activation queue for validators.
    /// - `spec`: Reference to the chain specification to determine effective balance increment.
    ///
    /// # Returns
    ///
    /// A new `EpochCache` instance.
    pub fn new(
        key: EpochCacheKey,
        effective_balances: Vec<u64>,
        base_rewards: Vec<u64>,
        activation_queue: ActivationQueue,
        spec: &ChainSpec,
    ) -> EpochCache {
        Self {
            inner: Some(Arc::new(Inner {
                key,
                effective_balances,
                base_rewards,
                activation_queue,
                effective_balance_increment: spec.effective_balance_increment,
            })),
        }
    }

    /// Checks the validity of the `EpochCache` against the current epoch and state's decision root.
    ///
    /// # Parameters
    ///
    /// - `current_epoch`: The current epoch number.
    /// - `state_decision_root`: The decision block root of the current state.
    ///
    /// # Returns
    ///
    /// `Result<(), EpochCacheError>`:
    /// - `Ok(())` if the cache is valid.
    /// - `Err(EpochCacheError)` if there's a validation error.
    pub fn check_validity(
        &self,
        current_epoch: Epoch,
        state_decision_root: Hash256,
    ) -> Result<(), EpochCacheError> {
        let cache = self
            .inner
            .as_ref()
            .ok_or(EpochCacheError::CacheNotInitialized)?;
        if cache.key.epoch != current_epoch {
            return Err(EpochCacheError::IncorrectEpoch {
                cache: cache.key.epoch,
                state: current_epoch,
            });
        }
        if cache.key.decision_block_root != state_decision_root {
            return Err(EpochCacheError::IncorrectDecisionBlock {
                cache: cache.key.decision_block_root,
                state: state_decision_root,
            });
        }
        Ok(())
    }

    /// Retrieves the effective balance for a validator given its index.
    ///
    /// # Parameters
    ///
    /// - `validator_index`: Index of the validator.
    ///
    /// # Returns
    ///
    /// `Result<u64, EpochCacheError>`:
    /// - `Ok(u64)` with the effective balance if found.
    /// - `Err(EpochCacheError)` if the index is out of bounds.
    #[inline]
    pub fn get_effective_balance(&self, validator_index: usize) -> Result<u64, EpochCacheError> {
        self.inner
            .as_ref()
            .ok_or(EpochCacheError::CacheNotInitialized)?
            .effective_balances
            .get(validator_index)
            .copied()
            .ok_or(EpochCacheError::ValidatorIndexOutOfBounds { validator_index })
    }

    /// Retrieves the base reward for a validator given its index.
    ///
    /// # Parameters
    ///
    /// - `validator_index`: Index of the validator.
    ///
    /// # Returns
    ///
    /// `Result<u64, EpochCacheError>`:
    /// - `Ok(u64)` with the base reward if found.
    /// - `Err(EpochCacheError)` if the index is out of bounds or if there's an arithmetic error.
    #[inline]
    pub fn get_base_reward(&self, validator_index: usize) -> Result<u64, EpochCacheError> {
        let inner = self
            .inner
            .as_ref()
            .ok_or(EpochCacheError::CacheNotInitialized)?;
        let effective_balance = self.get_effective_balance(validator_index)?;
        let effective_balance_eth =
            effective_balance.safe_div(inner.effective_balance_increment)? as usize;
        inner
            .base_rewards
            .get(effective_balance_eth)
            .copied()
            .ok_or(EpochCacheError::EffectiveBalanceOutOfBounds {
                effective_balance_eth,
            })
    }

    /// Retrieves a reference to the activation queue stored in the `EpochCache`.
    ///
    /// # Returns
    ///
    /// `Result<&ActivationQueue, EpochCacheError>`:
    /// - `Ok(&ActivationQueue)` if the cache is initialized.
    /// - `Err(EpochCacheError)` if the cache is not initialized.
    pub fn activation_queue(&self) -> Result<&ActivationQueue, EpochCacheError> {
        let inner = self
            .inner
            .as_ref()
            .ok_or(EpochCacheError::CacheNotInitialized)?;
        Ok(&inner.activation_queue)
    }
}
