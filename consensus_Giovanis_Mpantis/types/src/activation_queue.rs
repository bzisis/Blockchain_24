use crate::{ChainSpec, Epoch, Validator};
use std::collections::BTreeSet;

/// Activation queue computed during epoch processing for use in the *next* epoch.
#[derive(Debug, PartialEq, Eq, Default, Clone, arbitrary::Arbitrary)]
pub struct ActivationQueue {
    /// Validators represented by `(activation_eligibility_epoch, index)` in sorted order.
    ///
    /// These validators are not *necessarily* going to be activated. Their activation depends
    /// on how finalization is updated, and the `churn_limit`.
    queue: BTreeSet<(Epoch, usize)>,
}

impl ActivationQueue {
    /// Check if a validator could be eligible for activation in the next epoch and add them to
    /// the tentative activation queue if this is the case.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the validator.
    /// - `validator`: Reference to the validator object.
    /// - `next_epoch`: Epoch for which eligibility is being checked.
    /// - `spec`: Reference to the chain specification.
    ///
    /// This function checks if the validator `validator` could be eligible for activation in
    /// the epoch `next_epoch` based on the chain specification `spec`. If eligible, it adds
    /// the validator to the `ActivationQueue`.
    pub fn add_if_could_be_eligible_for_activation(
        &mut self,
        index: usize,
        validator: &Validator,
        next_epoch: Epoch,
        spec: &ChainSpec,
    ) {
        if validator.could_be_eligible_for_activation_at(next_epoch, spec) {
            self.queue
                .insert((validator.activation_eligibility_epoch, index));
        }
    }

    /// Determine the final activation queue after accounting for finalization & the churn limit.
    ///
    /// # Parameters
    ///
    /// - `finalized_epoch`: The epoch up to which finalization is considered.
    /// - `churn_limit`: The maximum number of validators that can be activated.
    ///
    /// # Returns
    ///
    /// A `BTreeSet` of indices representing validators eligible for activation, limited by
    /// `churn_limit`.
    ///
    /// This function filters the validators in the `ActivationQueue` based on their
    /// `eligibility_epoch` compared to `finalized_epoch` and returns up to `churn_limit`
    /// indices of validators that are eligible for activation.
    pub fn get_validators_eligible_for_activation(
        &self,
        finalized_epoch: Epoch,
        churn_limit: usize,
    ) -> BTreeSet<usize> {
        self.queue
            .iter()
            .filter_map(|&(eligibility_epoch, index)| {
                (eligibility_epoch <= finalized_epoch).then_some(index)
            })
            .take(churn_limit)
            .collect()
    }
}
