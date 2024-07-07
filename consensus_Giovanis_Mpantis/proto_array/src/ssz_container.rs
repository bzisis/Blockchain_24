use crate::proto_array::ProposerBoost;
use crate::{
    proto_array::{ProtoArray, ProtoNodeV16, ProtoNodeV17},
    proto_array_fork_choice::{ElasticList, ProtoArrayForkChoice, VoteTracker},
    Error, JustifiedBalances,
};
use ssz::{four_byte_option_impl, Encode};
use ssz_derive::{Decode, Encode};
use std::collections::HashMap;
use superstruct::superstruct;
use types::{Checkpoint, Hash256};

// Define a "legacy" implementation of `Option<usize>` which uses four bytes for encoding the union
// selector.
four_byte_option_impl!(four_byte_option_checkpoint, Checkpoint);

/// Alias for `SszContainerV17`.
pub type SszContainer = SszContainerV17;

/// Struct representing an SszContainer, with variants for different protocol versions.
#[superstruct(
    variants(V16, V17),
    variant_attributes(derive(Encode, Decode)),
    no_enum
)]
pub struct SszContainer {
    /// Vector of vote trackers.
    pub votes: Vec<VoteTracker>,
    /// Vector of balances.
    pub balances: Vec<u64>,
    /// Prune threshold value.
    pub prune_threshold: usize,
    /// Justified checkpoint.
    pub justified_checkpoint: Checkpoint,
    /// Finalized checkpoint.
    pub finalized_checkpoint: Checkpoint,
    /// Nodes vector, specific to V16 variant.
    #[superstruct(only(V16))]
    pub nodes: Vec<ProtoNodeV16>,
    /// Nodes vector, specific to V17 variant.
    #[superstruct(only(V17))]
    pub nodes: Vec<ProtoNodeV17>,
    /// Indices vector, containing tuples of Hash256 and usize.
    pub indices: Vec<(Hash256, usize)>,
    /// Previous proposer boost.
    pub previous_proposer_boost: ProposerBoost,
}

/// Try conversion from `SszContainerV16` to `SszContainer`.
impl TryInto<SszContainer> for SszContainerV16 {
    type Error = Error;

    /// Attempts to convert `SszContainerV16` into `SszContainer`.
    fn try_into(self) -> Result<SszContainer, Error> {
        let nodes: Result<Vec<ProtoNodeV17>, Error> =
            self.nodes.into_iter().map(TryInto::try_into).collect();

        Ok(SszContainer {
            votes: self.votes,
            balances: self.balances,
            prune_threshold: self.prune_threshold,
            justified_checkpoint: self.justified_checkpoint,
            finalized_checkpoint: self.finalized_checkpoint,
            nodes: nodes?,
            indices: self.indices,
            previous_proposer_boost: self.previous_proposer_boost,
        })
    }
}

/// Conversion from `SszContainer` to `SszContainerV16`.
impl From<SszContainer> for SszContainerV16 {
    /// Converts `SszContainer` into `SszContainerV16`.
    fn from(from: SszContainer) -> SszContainerV16 {
        let nodes = from.nodes.into_iter().map(Into::into).collect();

        SszContainerV16 {
            votes: from.votes,
            balances: from.balances,
            prune_threshold: from.prune_threshold,
            justified_checkpoint: from.justified_checkpoint,
            finalized_checkpoint: from.finalized_checkpoint,
            nodes,
            indices: from.indices,
            previous_proposer_boost: from.previous_proposer_boost,
        }
    }
}

/// Conversion from `ProtoArrayForkChoice` to `SszContainer`.
impl From<&ProtoArrayForkChoice> for SszContainer {
    /// Converts `ProtoArrayForkChoice` reference into `SszContainer`.
    fn from(from: &ProtoArrayForkChoice) -> Self {
        let proto_array = &from.proto_array;

        Self {
            votes: from.votes.0.clone(),
            balances: from.balances.effective_balances.clone(),
            prune_threshold: proto_array.prune_threshold,
            justified_checkpoint: proto_array.justified_checkpoint,
            finalized_checkpoint: proto_array.finalized_checkpoint,
            nodes: proto_array.nodes.clone(),
            indices: proto_array.indices.iter().map(|(k, v)| (*k, *v)).collect(),
            previous_proposer_boost: proto_array.previous_proposer_boost,
        }
    }
}

/// Try conversion from `SszContainer` to `ProtoArrayForkChoice`.
impl TryFrom<SszContainer> for ProtoArrayForkChoice {
    type Error = Error;

    /// Attempts to convert `SszContainer` into `ProtoArrayForkChoice`.
    fn try_from(from: SszContainer) -> Result<Self, Error> {
        let proto_array = ProtoArray {
            prune_threshold: from.prune_threshold,
            justified_checkpoint: from.justified_checkpoint,
            finalized_checkpoint: from.finalized_checkpoint,
            nodes: from.nodes,
            indices: from.indices.into_iter().collect::<HashMap<_, _>>(),
            previous_proposer_boost: from.previous_proposer_boost,
        };

        Ok(Self {
            proto_array,
            votes: ElasticList(from.votes),
            balances: JustifiedBalances::from_effective_balances(from.balances)?,
        })
    }
}
