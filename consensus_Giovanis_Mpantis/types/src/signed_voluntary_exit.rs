use crate::{test_utils::TestRandom, VoluntaryExit};
use bls::Signature;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a signed voluntary exit.
///
/// A `SignedVoluntaryExit` contains a `VoluntaryExit` message and a `Signature`
/// that authenticates the voluntary exit. This is used by validators who wish
/// to voluntarily exit the validator set.
///
/// # Spec
/// Version 0.12.1
///
/// # Fields
/// - `message`: The `VoluntaryExit` message indicating the validator's intent to exit.
/// - `signature`: The `Signature` of the validator, proving the authenticity of the exit message.
#[derive(
    arbitrary::Arbitrary,
    Debug,
    PartialEq,
    Hash,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct SignedVoluntaryExit {
    /// The `VoluntaryExit` message indicating the validator's intent to exit.
    pub message: VoluntaryExit,
    
    /// The `Signature` of the validator, proving the authenticity of the exit message.
    pub signature: Signature,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs SSZ (Simple Serialize) and Tree Hash tests on the `SignedVoluntaryExit` struct.
    ///
    /// This macro generates tests to ensure that the `SignedVoluntaryExit` struct
    /// can be correctly serialized, deserialized, and hashed according to the SSZ and
    /// Tree Hash specifications.
    ssz_and_tree_hash_tests!(SignedVoluntaryExit);
}
