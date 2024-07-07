use crate::test_utils::TestRandom;
use crate::*;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a signed BLS to execution change.
///
/// This struct contains a BLS to execution change message along with its corresponding
/// cryptographic signature. It is used to authenticate the transition of a validator's
/// credentials from BLS (Boneh-Lynn-Shacham) to an execution address.
///
/// # Derives
/// - `Arbitrary`: Allows generating arbitrary instances of the struct for testing.
/// - `Debug`: Enables formatting the struct using the `{:?}` formatter.
/// - `PartialEq`, `Eq`: Allows comparing instances of the struct for equality.
/// - `Hash`: Enables hashing of the struct.
/// - `Clone`: Allows cloning instances of the struct.
/// - `Serialize`, `Deserialize`: Supports serialization and deserialization of the struct.
/// - `Encode`, `Decode`: Enables encoding and decoding of the struct for SSZ (Simple Serialize).
/// - `TreeHash`: Allows computing the Merkle tree hash of the struct.
/// - `TestRandom`: Enables generating random instances for testing purposes.
#[derive(
    arbitrary::Arbitrary,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct SignedBlsToExecutionChange {
    /// The BLS to execution change message.
    ///
    /// This field contains the details of the change, such as the new execution address
    /// and the validator's public key.
    pub message: BlsToExecutionChange,

    /// The cryptographic signature of the message.
    ///
    /// This signature is used to verify the authenticity and integrity of the message.
    pub signature: Signature,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests for the `SignedBlsToExecutionChange` struct.
    ///
    /// This module contains tests to ensure the correct functionality of the struct,
    /// including serialization, deserialization, and hashing.
    ssz_and_tree_hash_tests!(SignedBlsToExecutionChange);
}
