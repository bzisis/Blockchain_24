use crate::{
    light_client_update::*, test_utils::TestRandom, BeaconState, ChainSpec, EthSpec, FixedVector,
    ForkName, ForkVersionDeserialize, Hash256, LightClientHeader, LightClientHeaderAltair,
    LightClientHeaderCapella, LightClientHeaderDeneb, SignedBeaconBlock, Slot, SyncCommittee,
};
use derivative::Derivative;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use std::sync::Arc;
use superstruct::superstruct;
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// A LightClientBootstrap is the initializer sent to light_client nodes
/// trying to generate their basic storage when booting up.
#[superstruct(
    variants(Altair, Capella, Deneb),
    variant_attributes(
        derive(
            Debug,
            Clone,
            PartialEq,
            Serialize,
            Deserialize,
            Derivative,
            Decode,
            Encode,
            TestRandom,
            arbitrary::Arbitrary,
            TreeHash,
        ),
        serde(bound = "E: EthSpec", deny_unknown_fields),
        arbitrary(bound = "E: EthSpec"),
    )
)]
#[derive(
    Debug, Clone, Serialize, TreeHash, Encode, Deserialize, arbitrary::Arbitrary, PartialEq,
)]
#[serde(untagged)]
#[tree_hash(enum_behaviour = "transparent")]
#[ssz(enum_behaviour = "transparent")]
#[serde(bound = "E: EthSpec", deny_unknown_fields)]
#[arbitrary(bound = "E: EthSpec")]
pub struct LightClientBootstrap<E: EthSpec> {
    /// The requested beacon block header.
    #[superstruct(only(Altair), partial_getter(rename = "header_altair"))]
    pub header: LightClientHeaderAltair<E>,
    #[superstruct(only(Capella), partial_getter(rename = "header_capella"))]
    pub header: LightClientHeaderCapella<E>,
    #[superstruct(only(Deneb), partial_getter(rename = "header_deneb"))]
    pub header: LightClientHeaderDeneb<E>,
    /// The `SyncCommittee` used in the requested period.
    pub current_sync_committee: Arc<SyncCommittee<E>>,
    /// Merkle proof for sync committee
    pub current_sync_committee_branch: FixedVector<Hash256, CurrentSyncCommitteeProofLen>,
}

impl<E: EthSpec> LightClientBootstrap<E> {
    /// Maps a function over the variant of the bootstrap based on its fork name.
    pub fn map_with_fork_name<F, R>(&self, func: F) -> R
    where
        F: Fn(ForkName) -> R,
    {
        match self {
            Self::Altair(_) => func(ForkName::Altair),
            Self::Capella(_) => func(ForkName::Capella),
            Self::Deneb(_) => func(ForkName::Deneb),
        }
    }

    /// Retrieves the slot from the bootstrap.
    pub fn get_slot<'a>(&'a self) -> Slot {
        map_light_client_bootstrap_ref!(&'a _, self.to_ref(), |inner, cons| {
            cons(inner);
            inner.header.beacon.slot
        })
    }

    /// Constructs a `LightClientBootstrap` from SSZ bytes based on the fork name.
    pub fn from_ssz_bytes(bytes: &[u8], fork_name: ForkName) -> Result<Self, ssz::DecodeError> {
        let bootstrap = match fork_name {
            ForkName::Altair | ForkName::Bellatrix => {
                Self::Altair(LightClientBootstrapAltair::from_ssz_bytes(bytes)?)
            }
            ForkName::Capella => Self::Capella(LightClientBootstrapCapella::from_ssz_bytes(bytes)?),
            ForkName::Deneb | ForkName::Electra => {
                Self::Deneb(LightClientBootstrapDeneb::from_ssz_bytes(bytes)?)
            }
            ForkName::Base => {
                return Err(ssz::DecodeError::BytesInvalid(format!(
                    "LightClientBootstrap decoding for {fork_name} not implemented"
                )))
            }
        };

        Ok(bootstrap)
    }

    /// Calculates the maximum serialized length for SSZ based on the fork name.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn ssz_max_len_for_fork(fork_name: ForkName) -> usize {
        // TODO(electra): review electra changes
        match fork_name {
            ForkName::Base => 0,
            ForkName::Altair
            | ForkName::Bellatrix
            | ForkName::Capella
            | ForkName::Deneb
            | ForkName::Electra => {
                <LightClientBootstrapAltair<E> as Encode>::ssz_fixed_len()
                    + LightClientHeader::<E>::ssz_max_var_len_for_fork(fork_name)
            }
        }
    }

    /// Constructs a `LightClientBootstrap` from a `BeaconState` and a `SignedBeaconBlock`.
    pub fn from_beacon_state(
        beacon_state: &mut BeaconState<E>,
        block: &SignedBeaconBlock<E>,
        chain_spec: &ChainSpec,
    ) -> Result<Self, Error> {
        let mut header = beacon_state.latest_block_header().clone();
        header.state_root = beacon_state.update_tree_hash_cache()?;
        let current_sync_committee_branch =
            FixedVector::new(beacon_state.compute_merkle_proof(CURRENT_SYNC_COMMITTEE_INDEX)?)?;

        let current_sync_committee = beacon_state.current_sync_committee()?.clone();

        let light_client_bootstrap = match block
            .fork_name(chain_spec)
            .map_err(|_| Error::InconsistentFork)?
        {
            ForkName::Base => return Err(Error::AltairForkNotActive),
            ForkName::Altair | ForkName::Bellatrix => Self::Altair(LightClientBootstrapAltair {
                header: LightClientHeaderAltair::block_to_light_client_header(block)?,
                current_sync_committee,
                current_sync_committee_branch,
            }),
            ForkName::Capella => Self::Capella(LightClientBootstrapCapella {
                header: LightClientHeaderCapella::block_to_light_client_header(block)?,
                current_sync_committee,
                current_sync_committee_branch,
            }),
            ForkName::Deneb | ForkName::Electra => Self::Deneb(LightClientBootstrapDeneb {
                header: LightClientHeaderDeneb::block_to_light_client_header(block)?,
                current_sync_committee,
                current_sync_committee_branch,
            }),
        };

        Ok(light_client_bootstrap)
    }
}

impl<E: EthSpec> ForkVersionDeserialize for LightClientBootstrap<E> {
    /// Deserializes a `LightClientBootstrap` based on a fork name and a JSON value.
    fn deserialize_by_fork<'de, D: Deserializer<'de>>(
        value: Value,
        fork_name: ForkName,
    ) -> Result<Self, D::Error> {
        match fork_name {
            ForkName::Base => Err(serde::de::Error::custom(format!(
                "LightClientBootstrap failed to deserialize: unsupported fork '{}'",
                fork_name
            ))),
            _ => Ok(serde_json::from_value::<LightClientBootstrap<E>>(value)
                .map_err(serde::de::Error::custom))?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MainnetEthSpec;

    ssz_tests!(LightClientBootstrapDeneb<MainnetEthSpec>);
}
