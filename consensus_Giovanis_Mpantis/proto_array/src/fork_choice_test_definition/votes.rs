use super::*;

/// Generates a `ForkChoiceTestDefinition` for testing purposes.
pub fn get_votes_test_definition() -> ForkChoiceTestDefinition {
    let mut balances = vec![1; 2];
    let mut ops = vec![];

    // Ensure that the head starts at the finalized block.
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(0),
    });

    // Add a block with a hash of 2.
    //
    //          0
    //         /
    //        2
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(1),
        root: get_root(2),
        parent_root: get_root(0),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
    });

    // Ensure that the head is 2
    //
    //          0
    //         /
    // head-> 2
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(2),
    });

    // Add a block with a hash of 1 that comes off the genesis block (this is a fork compared
    // to the previous block).
    //
    //          0
    //         / \
    //        2   1
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(1),
        root: get_root(1),
        parent_root: get_root(0),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
    });

    // Ensure that the head is still 2
    //
    //          0
    //         / \
    // head-> 2   1
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(2),
    });

    // Add a vote to block 1
    //
    //          0
    //         / \
    //        2   1 <- +vote
    ops.push(Operation::ProcessAttestation {
        validator_index: 0,
        block_root: get_root(1),
        target_epoch: Epoch::new(2),
    });

    // Ensure that the head is now 1, because 1 has a vote.
    //
    //          0
    //         / \
    //        2   1 <- head
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(1),
    });

    // Add a vote to block 2
    //
    //           0
    //          / \
    // +vote-> 2   1
    ops.push(Operation::ProcessAttestation {
        validator_index: 1,
        block_root: get_root(2),
        target_epoch: Epoch::new(2),
    });

    // Ensure that the head is 2 since 1 and 2 both have a vote
    //
    //          0
    //         / \
    // head-> 2   1
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(2),
    });

    // Add block 3.
    //
    //          0
    //         / \
    //        2   1
    //            |
    //            3
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(2),
        root: get_root(3),
        parent_root: get_root(1),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
    });

    // Ensure that the head is still 2
    //
    //          0
    //         / \
    // head-> 2   1
    //            |
    //            3
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(2),
    });

    // Move validator #0 vote from 1 to 3
    //
    //          0
    //         / \
    //        2   1 <- -vote
    //            |
    //            3 <- +vote
    ops.push(Operation::ProcessAttestation {
        validator_index: 0,
        block_root: get_root(3),
        target_epoch: Epoch::new(3),
    });

    // Ensure that the head is still 2
    //
    //          0
    //         / \
    // head-> 2   1
    //            |
    //            3
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(2),
    });

    // Move validator #1 vote from 2 to 1 (this is an equivocation, but fork choice doesn't
    // care)
    //
    //           0
    //          / \
    // -vote-> 2   1 <- +vote
    //             |
    //             3
    ops.push(Operation::ProcessAttestation {
        validator_index: 1,
        block_root: get_root(1),
        target_epoch: Epoch::new(3),
    });

    // Ensure that the head is now 3
    //
    //          0
    //         / \
    //        2   1
    //            |
    //            3 <- head
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(3),
    });

    // Add block 4.
    //
    //          0
    //         / \
    //        2   1
    //            |
    //            3
    //            |
    //            4
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(3),
        root: get_root(4),
        parent_root: get_root(3),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
    });

    // Ensure that the head is now 4
    //
    //          0
    //         / \
    //        2   1
    //            |
    //            3
    //            |
    //            4 <- head
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(4),
    });

    // Add block 5, which has a justified epoch of 2.
    //
    //          0
    //         / \
    //        2   1
    //            |
    //            3
    //            |
    //            4
    //           /
    //          5 <- justified epoch = 2
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(4),
        root: get_root(5),
        parent_root: get_root(4),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(1),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(1),
        },
    });

    // Ensure that 5 is filtered out and the head stays at 4.
    //
    //          0
    //         / \
    //        2   1
    //            |
    //            3
    //            |
    //            4 <- head
    //           /
    //          5
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(4),
    });

    // Return the generated `ForkChoiceTestDefinition`.
    ForkChoiceTestDefinition { ops }
}
/// Pushes an operation to find the head of the blockchain, based on given checkpoints,
/// state balances, and an expected head.
///
/// # Arguments
///
/// * `justified_checkpoint` - The justified checkpoint containing epoch and root.
/// * `finalized_checkpoint` - The finalized checkpoint containing epoch and root.
/// * `justified_state_balances` - Cloned state balances.
/// * `expected_head` - The expected root of the head block.
ops.push(Operation::FindHead {
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(1),
        root: get_root(0),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(1),
        root: get_root(0),
    },
    justified_state_balances: balances.clone(),
    expected_head: get_root(4),
});

/// Pushes an operation to process a block with given slot, root, parent root, and checkpoints.
///
/// # Arguments
///
/// * `slot` - The slot of the block.
/// * `root` - The root of the block.
/// * `parent_root` - The root of the parent block.
/// * `justified_checkpoint` - The justified checkpoint containing epoch and root.
/// * `finalized_checkpoint` - The finalized checkpoint containing epoch and root.
ops.push(Operation::ProcessBlock {
    slot: Slot::new(0),
    root: get_root(6),
    parent_root: get_root(4),
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(1),
        root: get_root(0),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(1),
        root: get_root(0),
    },
});

/// Pushes an operation to process an attestation by a validator for a specific block and epoch.
///
/// # Arguments
///
/// * `validator_index` - Index of the validator attesting.
/// * `block_root` - Root of the block being attested.
/// * `target_epoch` - Target epoch for the attestation.
ops.push(Operation::ProcessAttestation {
    validator_index: 0,
    block_root: get_root(5),
    target_epoch: Epoch::new(4),
});

ops.push(Operation::ProcessAttestation {
    validator_index: 1,
    block_root: get_root(5),
    target_epoch: Epoch::new(4),
});

/// Pushes operations to process blocks 7, 8, and 9 with specified roots and checkpoints.
///
/// # Arguments
///
/// Each block operation includes:
/// * `slot` - The slot of the block.
/// * `root` - The root of the block.
/// * `parent_root` - The root of the parent block.
/// * `justified_checkpoint` - The justified checkpoint containing epoch and root.
/// * `finalized_checkpoint` - The finalized checkpoint containing epoch and root.
ops.push(Operation::ProcessBlock {
    slot: Slot::new(0),
    root: get_root(7),
    parent_root: get_root(5),
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
});

ops.push(Operation::ProcessBlock {
    slot: Slot::new(0),
    root: get_root(8),
    parent_root: get_root(7),
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
});

ops.push(Operation::ProcessBlock {
    slot: Slot::new(0),
    root: get_root(9),
    parent_root: get_root(8),
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
});

/// Pushes an operation to find the head of the blockchain, ensuring that block 6 is the expected head.
///
/// # Arguments
///
/// * `justified_checkpoint` - The justified checkpoint containing epoch and root.
/// * `finalized_checkpoint` - The finalized checkpoint containing epoch and root.
/// * `justified_state_balances` - Cloned state balances.
/// * `expected_head` - The expected root of the head block.
ops.push(Operation::FindHead {
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(1),
        root: get_root(0),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(1),
        root: get_root(0),
    },
    justified_state_balances: balances.clone(),
    expected_head: get_root(6),
});

/// Pushes an operation to find the head of the blockchain, ensuring that block 9 is the expected head
/// after updating the justified epoch to 2.
///
/// # Arguments
///
/// * `justified_checkpoint` - The justified checkpoint containing epoch and root.
/// * `finalized_checkpoint` - The finalized checkpoint containing epoch and root.
/// * `justified_state_balances` - Cloned state balances.
/// * `expected_head` - The expected root of the head block.
ops.push(Operation::FindHead {
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    justified_state_balances: balances.clone(),
    expected_head: get_root(9),
});

/// Pushes operations to process attestations from two validators for block 9, targeting epoch 5.
///
/// # Arguments
///
/// * `validator_index` - Index of the validator attesting.
/// * `block_root` - Root of the block being attested.
/// * `target_epoch` - Target epoch for the attestation.
ops.push(Operation::ProcessAttestation {
    validator_index: 0,
    block_root: get_root(9),
    target_epoch: Epoch::new(5),
});

ops.push(Operation::ProcessAttestation {
    validator_index: 1,
    block_root: get_root(9),
    target_epoch: Epoch::new(5),
});

/// Pushes an operation to process block 10 with specified root and checkpoints.
///
/// # Arguments
///
/// * `slot` - The slot of the block.
/// * `root` - The root of the block.
/// * `parent_root` - The root of the parent block.
/// * `justified_checkpoint` - The justified checkpoint containing epoch and root.
/// * `finalized_checkpoint` - The finalized checkpoint containing epoch and root.
ops.push(Operation::ProcessBlock {
    slot: Slot::new(0),
    root: get_root(10),
    parent_root: get_root(8),
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
});

/// Pushes an operation to find the head of the blockchain, ensuring that block 10 is the expected head.
///
/// # Arguments
///
/// * `justified_checkpoint` - The justified checkpoint containing epoch and root.
/// * `finalized_checkpoint` - The finalized checkpoint containing epoch and root.
/// * `justified_state_balances` - Cloned state balances.
/// * `expected_head` - The expected root of the head block.
ops.push(Operation::FindHead {
    justified_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    finalized_checkpoint: Checkpoint {
        epoch: Epoch::new(2),
        root: get_root(5),
    },
    justified_state_balances: balances.clone(),
    expected_head: get_root(10),
});
/// Constructs a test definition for fork choice based on operations.
fn get_votes_test_definition() -> ForkChoiceTestDefinition {
    let mut ops = Vec::new();
    let mut balances;

    // Set the balances of the last two validators to zero
    balances = vec![1, 1, 0, 0];

    // Check the head is 9 again.
    //
    //          .
    //          .
    //          .
    //          |
    //          8
    //         / \
    // head-> 9  10
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Set the balances of the last two validators back to 1
    balances = vec![1; 4];

    // Check the head is 10.
    //
    //          .
    //          .
    //          .
    //          |
    //          8
    //         / \
    //        9  10 <- head
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(10),
    });

    // Remove the last two validators
    balances = vec![1; 2];

    // Check the head is 9 again.
    //
    //  (prior blocks omitted for brevity)
    //          .
    //          .
    //          .
    //          |
    //          8
    //         / \
    // head-> 9  10
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Ensure that pruning below the prune threshold does not prune.
    ops.push(Operation::Prune {
        finalized_root: get_root(5),
        prune_threshold: usize::MAX,
        expected_len: 11,
    });

    // Run find-head, ensure the no-op prune didn't change the head.
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Ensure that pruning above the prune threshold does prune.
    //
    //
    //          0
    //         / \
    //        2   1
    //            |
    //            3
    //            |
    //            4
    // -------pruned here ------
    //          5   6
    //          |
    //          7
    //          |
    //          8
    //         / \
    //        9  10
    ops.push(Operation::Prune {
        finalized_root: get_root(5),
        prune_threshold: 1,
        expected_len: 6,
    });

    // Run find-head, ensure the prune didn't change the head.
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        justified_state_balances: balances.clone(),
        expected_head: get_root(9),
    });

    // Add block 11
    //
    //          5   6
    //          |
    //          7
    //          |
    //          8
    //         / \
    //        9  10
    //        |
    //        11
    ops.push(Operation::ProcessBlock {
        slot: Slot::new(0),
        root: get_root(11),
        parent_root: get_root(9),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
    });

    // Ensure the head is now 11
    //
    //          5   6
    //          |
    //          7
    //          |
    //          8
    //         / \
    //        9  10
    //        |
    // head-> 11
    ops.push(Operation::FindHead {
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(2),
            root: get_root(5),
        },
        justified_state_balances: balances,
        expected_head: get_root(11),
    });

    // Return the constructed test definition
    ForkChoiceTestDefinition {
        finalized_block_slot: Slot::new(0),
        justified_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        finalized_checkpoint: Checkpoint {
            epoch: Epoch::new(1),
            root: get_root(0),
        },
        operations: ops,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let test = get_votes_test_definition();
        test.run(); // Assuming run() method exists on ForkChoiceTestDefinition
    }
}
