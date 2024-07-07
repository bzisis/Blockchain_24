#![cfg(test)]

use crate::per_epoch_processing::process_epoch;
use beacon_chain::test_utils::BeaconChainHarness;
use beacon_chain::types::{EthSpec, MinimalEthSpec};
use bls::Hash256;
use env_logger::{Builder, Env};
use types::Slot;

/// Asynchronous test that checks if the main function runs without error.
#[tokio::test]
async fn runs_without_error() {
    // Initialize logging for error level messages.
    Builder::from_env(Env::default().default_filter_or("error")).init();

    // Setup BeaconChainHarness for testing.
    let harness = BeaconChainHarness::builder(MinimalEthSpec)
        .default_spec()
        .deterministic_keypairs(8)
        .fresh_ephemeral_store()
        .build();
    harness.advance_slot();

    // Setup variables for testing.
    let spec = MinimalEthSpec::default_spec();
    let target_slot =
        (MinimalEthSpec::genesis_epoch() + 4).end_slot(MinimalEthSpec::slots_per_epoch());

    let state = harness.get_current_state();

    // Add attested blocks at specific slots.
    harness
        .add_attested_blocks_at_slots(
            state,
            Hash256::zero(),
            (1..target_slot.as_u64())
                .map(Slot::new)
                .collect::<Vec<_>>()
                .as_slice(),
            (0..8).collect::<Vec<_>>().as_slice(),
        )
        .await;
    let mut new_head_state = harness.get_current_state();

    // Process epoch on the new state.
    process_epoch(&mut new_head_state, &spec).unwrap();
}

#[cfg(not(debug_assertions))]
mod release_tests {
    use super::*;
    use crate::{
        per_slot_processing::per_slot_processing,
        EpochProcessingError,
        SlotProcessingError,
    };
    use beacon_chain::test_utils::{
        AttestationStrategy,
        BlockStrategy,
    };
    use types::{
        Epoch,
        ForkName,
        InconsistentFork,
        MainnetEthSpec,
    };

    /// Asynchronous test for checking Altair state on base fork scenario.
    #[tokio::test]
    async fn altair_state_on_base_fork() {
        // Initialize the specification with Altair fork epoch.
        let mut spec = MainnetEthSpec::default_spec();
        let slots_per_epoch = MainnetEthSpec::slots_per_epoch();
        spec.altair_fork_epoch = Some(Epoch::new(1));

        // Generate Altair state for testing.
        let altair_state = {
            let harness = BeaconChainHarness::builder(MainnetEthSpec)
                .spec(spec.clone())
                .deterministic_keypairs(8)
                .fresh_ephemeral_store()
                .build();

            harness.advance_slot();

            harness
                .extend_chain(
                    // Extend the chain to reach Altair fork epoch.
                    (slots_per_epoch * 2 - 1) as usize,
                    BlockStrategy::OnCanonicalHead,
                    AttestationStrategy::AllValidators,
                )
                .await;

            harness.get_current_state()
        };

        // Pre-conditions for a valid test.
        assert_eq!(altair_state.fork_name(&spec).unwrap(), ForkName::Altair);
        assert_eq!(
            altair_state.slot(),
            altair_state.current_epoch().end_slot(slots_per_epoch)
        );

        // Check initial validity of Altair state.
        process_epoch(&mut altair_state.clone(), &spec)
            .expect("state passes initial epoch processing");
        per_slot_processing(&mut altair_state.clone(), None, &spec)
            .expect("state passes initial slot processing");

        // Modify the spec so Altair never happens.
        spec.altair_fork_epoch = None;

        // Define the expected error for consistency check.
        let expected_err = InconsistentFork {
            fork_at_slot: ForkName::Base,
            object_fork: ForkName::Altair,
        };

        // Assertions for the inconsistency scenario.
        assert_eq!(altair_state.fork_name(&spec), Err(expected_err));
        assert_eq!(
            process_epoch(&mut altair_state.clone(), &spec),
            Err(EpochProcessingError::InconsistentStateFork(expected_err))
        );
        assert_eq!(
            per_slot_processing(&mut altair_state.clone(), None, &spec),
            Err(SlotProcessingError::InconsistentStateFork(expected_err))
        );
    }

    /// Asynchronous test for checking base state on Altair fork scenario.
    #[tokio::test]
    async fn base_state_on_altair_fork() {
        // Initialize the specification with Altair fork epoch as None.
        let mut spec = MainnetEthSpec::default_spec();
        let slots_per_epoch = MainnetEthSpec::slots_per_epoch();
        spec.altair_fork_epoch = None;

        // Generate base state for testing.
        let base_state = {
            let harness = BeaconChainHarness::builder(MainnetEthSpec)
                .spec(spec.clone())
                .deterministic_keypairs(8)
                .fresh_ephemeral_store()
                .build();

            harness.advance_slot();

            harness
                .extend_chain(
                    // Extend the chain to a point where Altair fork would have occurred.
                    (slots_per_epoch * 2 - 1) as usize,
                    BlockStrategy::OnCanonicalHead,
                    AttestationStrategy::AllValidators,
                )
                .await;

            harness.get_current_state()
        };

        // Pre-conditions for a valid test.
        assert_eq!(base_state.fork_name(&spec).unwrap(), ForkName::Base);
        assert_eq!(
            base_state.slot(),
            base_state.current_epoch().end_slot(slots_per_epoch)
        );

        // Check initial validity of base state.
        process_epoch(&mut base_state.clone(), &spec)
            .expect("state passes initial epoch processing");
        per_slot_processing(&mut base_state.clone(), None, &spec)
            .expect("state passes initial slot processing");

        // Modify the spec so Altair happens at the first epoch.
        spec.altair_fork_epoch = Some(Epoch::new(1));

        // Define the expected error for consistency check.
        let expected_err = InconsistentFork {
            fork_at_slot: ForkName::Altair,
            object_fork: ForkName::Base,
        };

        // Assertions for the inconsistency scenario.
        assert_eq!(base_state.fork_name(&spec), Err(expected_err));
        assert_eq!(
            process_epoch(&mut base_state.clone(), &spec),
            Err(EpochProcessingError::InconsistentStateFork(expected_err))
        );
        assert_eq!(
            per_slot_processing(&mut base_state.clone(), None, &spec),
            Err(SlotProcessingError::InconsistentStateFork(expected_err))
        );
    }
}
