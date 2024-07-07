use super::*;

/// Constructs a ForkChoiceTestDefinition for the FFG case 01 test scenario.
///
/// Returns a `ForkChoiceTestDefinition` that specifies operations to test
/// fork choice under various justified and finalized epochs.
pub fn get_ffg_case_01_test_definition() -> ForkChoiceTestDefinition {
    let balances = vec![1; 2];
    let mut ops = vec![];

    // Ensure that the head starts at the finalized block.
    ops.push(Operation::FindHead {
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(0),
    });

    // Build the following tree (stick? lol).
    //
    //            0 <- just: 0, fin: 0
    //            |
    //            1 <- just: 0, fin: 0
    //            |
    //            2 <- just: 1, fin: 0
    //            |
    //            3 <- just: 2, fin: 1
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(1),
        root: get_root(1),
        parent_root: get_root(0),
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(2),
        root: get_root(2),
        parent_root: get_root(1),
        justified_checkpoint: get_checkpoint(1),
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(3),
        root: get_root(3),
        parent_root: get_root(2),
        justified_checkpoint: get_checkpoint(2),
        finalized_checkpoint: get_checkpoint(1),
    });

    // Ensure that with justified epoch 0 we find 3
    //
    //            0 <- start
    //            |
    //            1
    //            |
    //            2
    //            |
    //            3 <- head
    ops.push(Operation::FindHead {
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(3),
    });

    // Ensure that with justified epoch 1 we find 3
    //
    //            0
    //            |
    //            1
    //            |
    //            2 <- start
    //            |
    //            3 <- head
    //
    // Since https://github.com/ethereum/consensus-specs/pull/3431 it is valid
    // to elect head blocks that have a higher justified checkpoint than the
    // store.
    ops.push(Operation::FindHead {
        justified_checkpoint: get_checkpoint(1),
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(3),
    });

    // Ensure that with justified epoch 2 we find 3
    //
    //            0
    //            |
    //            1
    //            |
    //            2
    //            |
    //            3 <- start + head
    ops.push(Operation::FindHead {
        justified_checkpoint: get_checkpoint(2),
        finalized_checkpoint: get_checkpoint(1),
        justified_state_balances: balances,
        expected_head: get_root(3),
    });

    // END OF TESTS
    ForkChoiceTestDefinition {
        finalized_block_slot: Slot::new(0),
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
        operations: ops,
    }
}

/// Constructs a ForkChoiceTestDefinition for the FFG case 02 test scenario.
///
/// Returns a `ForkChoiceTestDefinition` that specifies operations to test
/// fork choice under various justified and finalized epochs.
pub fn get_ffg_case_02_test_definition() -> ForkChoiceTestDefinition {
    let balances = vec![1; 2];
    let mut ops = vec![];

    // Ensure that the head starts at the finalized block.
    ops.push(Operation::FindHead {
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(0),
    });

    // Build the following tree.
    //
    //                       0
    //                      / \
    //  just: 0, fin: 0 -> 1   2 <- just: 0, fin: 0
    //                     |   |
    //  just: 1, fin: 0 -> 3   4 <- just: 0, fin: 0
    //                     |   |
    //  just: 1, fin: 0 -> 5   6 <- just: 0, fin: 0
    //                     |   |
    //  just: 1, fin: 0 -> 7   8 <- just: 1, fin: 0
    //                     |   |
    //  just: 2, fin: 0 -> 9  10 <- just: 2, fin: 0

    // Left branch
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(1),
        root: get_root(1),
        parent_root: get_root(0),
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(2),
        root: get_root(3),
        parent_root: get_root(1),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(1),
        },
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(3),
        root: get_root(5),
        parent_root: get_root(3),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(1),
        },
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(4),
        root: get_root(7),
        parent_root: get_root(5),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(1),
        },
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(5),
        root: get_root(9),
        parent_root: get_root(7),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(3),
        },
        finalized_checkpoint: get_checkpoint(0),
    });

    // Right branch
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(1),
        root: get_root(2),
        parent_root: get_root(0),
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(2),
        root: get_root(4),
        parent_root: get_root(2),
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(3),
        root: get_root(6),
        parent_root: get_root(4),
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(4),
        root: get_root(8),
        parent_root: get_root(6),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(2),
        },
        finalized_checkpoint: get_checkpoint(0),
    });
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(5),
        root: get_root(10),
        parent_root: get_root(8),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(4),
        },
        finalized_checkpoint: get_checkpoint(0),
    });

    // Ensure that if we start at 0 we find 10 (just: 0, fin: 0).
    //
    //           0  <-- start
    //          / \
    //         1   2
    //         |   |
    //         3   4
    //         |   |
    //         5   6
    //         |   |
    //         7   8
    //         |   |
    //         9  10 <-- head
    ops.push(Operation::FindHead {
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(10),
    });
    // Same as above, but with justified epoch 2.
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(4),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(10),
    });
    // Same as above, but with justified epoch 3.
    //
    // Since https://github.com/ethereum/consensus-specs/pull/3431 it is valid
    // to elect head blocks that have a higher justified checkpoint than the
    // store.
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(3),
            root: get_root(6),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(10),
    });

    // Add a vote to 1.
    //
    //                 0
    //                / \
    //    +1 vote -> 1   2
    //               |   |
    //               3   4
    //               |   |
    //               5   6
    //               |   |
    //               7   8
    //               |   |
    //               9  10
    ops.push(Operation::ProcessAttestation {
        validator_index: 0,
        block_root: get_root(1),
        target_epoch: Epoch::new(0),
    });

    ForkChoiceTestDefinition {
        finalized_block_slot: Slot::new(0),
        justified_checkpoint: get_checkpoint(0),
        finalized_checkpoint: get_checkpoint(0),
        operations: ops,
    }
}
fn get_ffg_case_01_test_definition() -> ForkChoiceTestDefinition {
    // Ensure that if we start at 0 we find 9 (just: 0, fin: 0).
    //
    //           0  <-- start
    //          / \
    //         1   2
    //         |   |
    //         3   4
    //         |   |
    //         5   6
    //         |   |
    //         7   8
    // head -> 9  10
    // Define operations for finding head blocks.
    let mut ops = Vec::new();

    // First operation: justified epoch 0.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 0.
        justified_checkpoint: get_checkpoint(0),
        /// Finalized checkpoint for epoch 0.
        finalized_checkpoint: get_checkpoint(0),
        /// State balances at justified state.
        justified_state_balances: balances.clone(),
        /// Expected head root for epoch 0.
        expected_head: get_root(9),
    });

    // Second operation: justified epoch 2.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 2.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(3),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Third operation: justified epoch 3.
    //
    // Since https://github.com/ethereum/consensus-specs/pull/3431 it is valid
    // to elect head blocks that have a higher justified checkpoint than the
    // store.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 3.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(3),
            root: get_root(5),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Add a vote to block 2.
    //
    //                 0
    //                / \
    //               1   2 <- +1 vote
    //               |   |
    //               3   4
    //               |   |
    //               5   6
    //               |   |
    //               7   8
    //               |   |
    //               9  10
    ops.push(Operation::ProcessAttestation {
        /// Index of the validator.
        validator_index: 1,
        /// Root of the block to vote for.
        block_root: get_root(2),
        /// Target epoch of the attestation.
        target_epoch: Epoch::new(0),
    });

    // Ensure that if we start at 0 we find 10 (just: 0, fin: 0).
    //
    //           0  <-- start
    //          / \
    //         1   2
    //         |   |
    //         3   4
    //         |   |
    //         5   6
    //         |   |
    //         7   8
    //         |   |
    // head -> 9  10
    // Operations to find head blocks.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 0.
        justified_checkpoint: get_checkpoint(0),
        /// Finalized checkpoint for epoch 0.
        finalized_checkpoint: get_checkpoint(0),
        /// State balances at justified state.
        justified_state_balances: balances.clone(),
        /// Expected head root for epoch 0.
        expected_head: get_root(10),
    });

    // Same as above but justified epoch 2.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 2.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(4),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(10),
    });

    // Same as above but justified epoch 3.
    //
    // Since https://github.com/ethereum/consensus-specs/pull/3431 it is valid
    // to elect head blocks that have a higher justified checkpoint than the
    // store.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 3.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(3),
            root: get_root(6),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(10),
    });

    // Ensure that if we start at 1 we find 9 (just: 0, fin: 0).
    //
    //            0
    //           / \
    //  start-> 1   2
    //          |   |
    //          3   4
    //          |   |
    //          5   6
    //          |   |
    //          7   8
    //          |   |
    //  head -> 9  10
    // Operations to find head blocks.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 0.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(0),
            root: get_root(1),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Same as above but justified epoch 2.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 2.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(3),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Same as above but justified epoch 3.
    //
    // Since https://github.com/ethereum/consensus-specs/pull/3431 it is valid
    // to elect head blocks that have a higher justified checkpoint than the
    // store.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 3.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(3),
            root: get_root(5),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Ensure that if we start at 2 we find 10 (just: 0, fin: 0).
    //
    //            0
    //           / \
    //          1   2 <- start
    //          |   |
    //          3   4
    //          |   |
    //          5   6
    //          |   |
    //          7   8
    //          |   |
    //          9  10 <- head
    // Operations to find head blocks.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 0.
        justified_checkpoint: get_checkpoint(0),
        /// Finalized checkpoint for epoch 0.
        finalized_checkpoint: get_checkpoint(0),
        /// State balances at justified state.
        justified_state_balances: balances.clone(),
        /// Expected head root for epoch 0.
        expected_head: get_root(10),
    });

    // Same as above but justified epoch 2.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 2.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(4),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances.clone(),
        expected_head: get_root(10),
    });

    // Same as above but justified epoch 3.
    //
    // Since https://github.com/ethereum/consensus-specs/pull/3431 it is valid
    // to elect head blocks that have a higher justified checkpoint than the
    // store.
    ops.push(Operation::FindHead {
        /// Justified checkpoint for epoch 3.
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(3),
            root: get_root(6),
        },
        finalized_checkpoint: get_checkpoint(0),
        justified_state_balances: balances,
        expected_head: get_root(10),
    });

    // END OF TESTS
    // Define the test definition with finalized block slot, justified checkpoint,
    // finalized checkpoint, and operations.
    ForkChoiceTestDefinition {
        /// Finalized block slot.
        finalized_block_slot: Slot::new(0),
        /// Justified checkpoint.
        justified_checkpoint: get_checkpoint(0),
        /// Finalized checkpoint.
        finalized_checkpoint: get_checkpoint(0),
        /// Operations to be executed.
        operations: ops,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffg_case_01() {
        let test = get_ffg_case_01_test_definition();
        test.run();
    }

    #[test]
    fn ffg_case_02() {
        let test = get_ffg_case_02_test_definition();
        test.run();
    }
}
