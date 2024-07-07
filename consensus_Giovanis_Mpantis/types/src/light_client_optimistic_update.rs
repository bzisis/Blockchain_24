use super::{EthSpec, ForkName, ForkVersionDeserialize, LightClientHeader, Slot, SyncAggregate};
use crate::test_utils::TestRandom;
use crate::{
    light_client_update::*, ChainSpec, LightClientHeaderAltair, LightClientHeaderCapella,
    LightClientHeaderDeneb, SignedBeaconBlock,
};
use derivative::Derivative;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use superstruct::superstruct;
use test_random_derive::TestRandom;
use tree_hash::Hash256;
use tree_hash_derive::TreeHash;

/// A LightClientOptimisticUpdate is the update we send on each slot,
/// it is based off the current unfinalized epoch is verified only against BLS signature.
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
    Debug, Clone, Serialize, Encode, TreeHash, Deserialize, arbitrary::Arbitrary, PartialEq,
)]
#[serde(untagged)]
#[tree_hash(enum_behaviour = "transparent")]
#[ssz(enum_behaviour = "transparent")]
#[serde(bound = "E: EthSpec", deny_unknown_fields)]
#[arbitrary(bound = "E: EthSpec")]
pub struct LightClientOptimisticUpdate<E: EthSpec> {
    /// The last `BeaconBlockHeader` from the last attested block by the sync committee.
    #[superstruct(only(Altair), partial_getter(rename = "attested_header_altair"))]
    pub attested_header: LightClientHeaderAltair<E>,
    #[superstruct(only(Capella), partial_getter(rename = "attested_header_capella"))]
    pub attested_header: LightClientHeaderCapella<E>,
    #[superstruct(only(Deneb), partial_getter(rename = "attested_header_deneb"))]
    pub attested_header: LightClientHeaderDeneb<E>,
    /// current sync aggregate
    pub sync_aggregate: SyncAggregate<E>,
    /// Slot of the sync aggregated signature
    pub signature_slot: Slot,
}

impl<E: EthSpec> LightClientOptimisticUpdate<E> {
    /// Creates a new `LightClientOptimisticUpdate` based on the given attested block, sync aggregate, and signature slot.
    pub fn new(
        attested_block: &SignedBeaconBlock<E>,
        sync_aggregate: SyncAggregate<E>,
        signature_slot: Slot,
        chain_spec: &ChainSpec,
    ) -> Result<Self, Error> {
        let optimistic_update = match attested_block
            .fork_name(chain_spec)
            .map_err(|_| Error::InconsistentFork)?
        {
            ForkName::Altair | ForkName::Bellatrix => {
                Self::Altair(LightClientOptimisticUpdateAltair {
                    attested_header: LightClientHeaderAltair::block_to_light_client_header(
                        attested_block,
                    )?,
                    sync_aggregate,
                    signature_slot,
                })
            }
            ForkName::Capella => Self::Capella(LightClientOptimisticUpdateCapella {
                attested_header: LightClientHeaderCapella::block_to_light_client_header(
                    attested_block,
                )?,
                sync_aggregate,
                signature_slot,
            }),
            ForkName::Deneb | ForkName::Electra => Self::Deneb(LightClientOptimisticUpdateDeneb {
                attested_header: LightClientHeaderDeneb::block_to_light_client_header(
                    attested_block,
                )?,
                sync_aggregate,
                signature_slot,
            }),
            ForkName::Base => return Err(Error::AltairForkNotActive),
        };

        Ok(optimistic_update)
    }

    /// Maps a function based on the fork name of the update.
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

    /// Retrieves the slot of the optimistic update.
    pub fn get_slot(&self) -> Slot {
        match self {
            Self::Altair(inner) => inner.attested_header.beacon.slot,
            Self::Capella(inner) => inner.attested_header.beacon.slot,
            Self::Deneb(inner) => inner.attested_header.beacon.slot,
        }
    }

    /// Retrieves the canonical root of the optimistic update.
    pub fn get_canonical_root(&self) -> Hash256 {
        match self {
            Self::Altair(inner) => inner.attested_header.beacon.canonical_root(),
            Self::Capella(inner) => inner.attested_header.beacon.canonical_root(),
            Self::Deneb(inner) => inner.attested_header.beacon.canonical_root(),
        }
    }

    /// Retrieves the parent root of the optimistic update.
    pub fn get_parent_root(&self) -> Hash256 {
        match self {
            Self::Altair(inner) => inner.attested_header.beacon.parent_root,
            Self::Capella(inner) => inner.attested_header.beacon.parent_root,
            Self::Deneb(inner) => inner.attested_header.beacon.parent_root,
        }
    }

    /// Decodes a `LightClientOptimisticUpdate` from SSZ bytes for a specific fork name.
    pub fn from_ssz_bytes(bytes: &[u8], fork_name: ForkName) -> Result<Self, ssz::DecodeError> {
        let optimistic_update = match fork_name {
            ForkName::Altair | ForkName::Bellatrix => {
                Self::Altair(LightClientOptimisticUpdateAltair::from_ssz_bytes(bytes)?)
            }
            ForkName::Capella => {
                Self::Capella(LightClientOptimisticUpdateCapella::from_ssz_bytes(bytes)?)
            }
            ForkName::Deneb | ForkName::Electra => {
                Self::Deneb(LightClientOptimisticUpdateDeneb::from_ssz_bytes(bytes)?)
            }
            ForkName::Base => {
                return Err(ssz::DecodeError::BytesInvalid(format!(
                    "LightClientOptimisticUpdate decoding for {fork_name} not implemented"
                )))
            }
        };

        Ok(optimistic_update)
    }

    /// Calculates the maximum SSZ length for a `LightClientOptimisticUpdate` based on the fork name.
    pub fn ssz_max_len_for_fork(fork_name: ForkName) -> usize {
        match fork_name {
            ForkName::Base => 0,
            ForkName::Altair
            | ForkName::Bellatrix
            | ForkName::Capella
            | ForkName::Deneb
            | ForkName::Electra => {
                <LightClientOptimisticUpdateAltair<E> as Encode>::ssz_fixed_len()
                    + LightClientHeader::<E>::ssz_max_var_len_for_fork(fork_name)
            }
        }
    }
}

impl<E: EthSpec> ForkVersionDeserialize for LightClientOptimisticUpdate<E> {
    /// Deserializes a `LightClientOptimisticUpdate` based on a fork name and a JSON value.
    fn deserialize_by_fork<'de, D: Deserializer<'de>>(
        value: Value,
        fork_name: ForkName,
    ) -> Result<Self, D::Error> {
        match fork_name {
            ForkName::Base => Err(serde::de::Error::custom(format!(
                "LightClientOptimisticUpdate failed to deserialize: unsupported fork '{}'",
                fork_name
            ))),
            _ => Ok(
                serde_json::from_value::<LightClientOptimisticUpdate<E>>(value)
                    .map_err(serde::de::Error::custom),
            )?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MainnetEthSpec;

    ssz_tests!(LightClientOptimisticUpdateDeneb<MainnetEthSpec>);
}
