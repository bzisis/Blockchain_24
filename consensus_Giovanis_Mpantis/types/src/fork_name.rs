use crate::{ChainSpec, Epoch};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Enum representing different fork names in the BeaconChain protocol.
///
/// This enum defines the possible fork names in the BeaconChain protocol, facilitating
/// differentiation between different protocol upgrades.
///
/// # Examples
///
/// ```
/// use beacon_chain::{ForkName, ChainSpec, Epoch};
///
/// let base_spec = ChainSpec::default();
/// let altair_spec = ForkName::Altair.make_genesis_spec(base_spec.clone());
/// assert_eq!(altair_spec.altair_fork_epoch, Some(Epoch::new(0)));
/// ```
#[derive(
    Debug, Clone, Copy, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(try_from = "String")]
#[serde(into = "String")]
#[ssz(enum_behaviour = "tag")]
pub enum ForkName {
    /// The base protocol version (Phase 0).
    Base,
    /// The Altair upgrade fork.
    Altair,
    /// The Bellatrix upgrade fork (including merge).
    Bellatrix,
    /// The Capella upgrade fork.
    Capella,
    /// The Deneb upgrade fork.
    Deneb,
    /// The Electra upgrade fork.
    Electra,
}

impl ForkName {
    /// Returns a vector of all defined `ForkName` variants.
    pub fn list_all() -> Vec<ForkName> {
        vec![
            ForkName::Base,
            ForkName::Altair,
            ForkName::Bellatrix,
            ForkName::Capella,
            ForkName::Deneb,
            ForkName::Electra,
        ]
    }

    /// Returns the latest `ForkName` variant.
    ///
    /// # Panics
    ///
    /// Panics if no forks are defined.
    pub fn latest() -> ForkName {
        // This unwrap is safe as long as we have 1+ forks. It is tested below.
        *ForkName::list_all().last().unwrap()
    }

    /// Sets the activation slots in the given `ChainSpec` so that the fork named by `self`
    /// is the only fork in effect from genesis.
    pub fn make_genesis_spec(&self, mut spec: ChainSpec) -> ChainSpec {
        // Assumes GENESIS_EPOCH = 0, which is safe because it's a constant.
        match self {
            ForkName::Base => {
                spec.altair_fork_epoch = None;
                spec.bellatrix_fork_epoch = None;
                spec.capella_fork_epoch = None;
                spec.deneb_fork_epoch = None;
                spec.electra_fork_epoch = None;
                spec
            }
            ForkName::Altair => {
                spec.altair_fork_epoch = Some(Epoch::new(0));
                spec.bellatrix_fork_epoch = None;
                spec.capella_fork_epoch = None;
                spec.deneb_fork_epoch = None;
                spec.electra_fork_epoch = None;
                spec
            }
            ForkName::Bellatrix => {
                spec.altair_fork_epoch = Some(Epoch::new(0));
                spec.bellatrix_fork_epoch = Some(Epoch::new(0));
                spec.capella_fork_epoch = None;
                spec.deneb_fork_epoch = None;
                spec.electra_fork_epoch = None;
                spec
            }
            ForkName::Capella => {
                spec.altair_fork_epoch = Some(Epoch::new(0));
                spec.bellatrix_fork_epoch = Some(Epoch::new(0));
                spec.capella_fork_epoch = Some(Epoch::new(0));
                spec.deneb_fork_epoch = None;
                spec.electra_fork_epoch = None;
                spec
            }
            ForkName::Deneb => {
                spec.altair_fork_epoch = Some(Epoch::new(0));
                spec.bellatrix_fork_epoch = Some(Epoch::new(0));
                spec.capella_fork_epoch = Some(Epoch::new(0));
                spec.deneb_fork_epoch = Some(Epoch::new(0));
                spec.electra_fork_epoch = None;
                spec
            }
            ForkName::Electra => {
                spec.altair_fork_epoch = Some(Epoch::new(0));
                spec.bellatrix_fork_epoch = Some(Epoch::new(0));
                spec.capella_fork_epoch = Some(Epoch::new(0));
                spec.deneb_fork_epoch = Some(Epoch::new(0));
                spec.electra_fork_epoch = Some(Epoch::new(0));
                spec
            }
        }
    }

    /// Returns the name of the fork immediately prior to the current one.
    ///
    /// If `self` is `ForkName::Base` then `Base` is returned.
    pub fn previous_fork(self) -> Option<ForkName> {
        match self {
            ForkName::Base => None,
            ForkName::Altair => Some(ForkName::Base),
            ForkName::Bellatrix => Some(ForkName::Altair),
            ForkName::Capella => Some(ForkName::Bellatrix),
            ForkName::Deneb => Some(ForkName::Capella),
            ForkName::Electra => Some(ForkName::Deneb),
        }
    }

    /// Returns the name of the fork immediately after the current one.
    ///
    /// If `self` is the last known fork and has no successor, `None` is returned.
    pub fn next_fork(self) -> Option<ForkName> {
        match self {
            ForkName::Base => Some(ForkName::Altair),
            ForkName::Altair => Some(ForkName::Bellatrix),
            ForkName::Bellatrix => Some(ForkName::Capella),
            ForkName::Capella => Some(ForkName::Deneb),
            ForkName::Deneb => Some(ForkName::Electra),
            ForkName::Electra => None,
        }
    }
}

/// Implements conversion from a string to `ForkName`.
///
/// Allows converting a string representation of a fork name to its corresponding `ForkName` enum variant.
impl FromStr for ForkName {
    type Err = String;

    fn from_str(fork_name: &str) -> Result<Self, String> {
        Ok(match fork_name.to_lowercase().as_ref() {
            "phase0" | "base" => ForkName::Base,
            "altair" => ForkName::Altair,
            "bellatrix" | "merge" => ForkName::Bellatrix,
            "capella" => ForkName::Capella,
            "deneb" => ForkName::Deneb,
            "electra" => ForkName::Electra,
            _ => return Err(format!("unknown fork name: {}", fork_name)),
        })
    }
}

/// Implements display formatting for `ForkName`.
///
/// Provides a human-readable string representation of a `ForkName` variant.
impl Display for ForkName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ForkName::Base => "phase0".fmt(f),
            ForkName::Altair => "altair".fmt(f),
            ForkName::Bellatrix => "bellatrix".fmt(f),
            ForkName::Capella => "capella".fmt(f),
            ForkName::Deneb => "deneb".fmt(f),
            ForkName::Electra => "electra".fmt(f),
        }
    }
}

/// Implements conversion from `ForkName` to `String`.
///
/// Converts a `ForkName` enum variant to its corresponding string representation.
impl From<ForkName> for String {
    fn from(fork: ForkName) -> String {
        fork.to_string()
    }
}

/// Implements conversion from `String` to `ForkName`.
///
/// Allows converting a string representation of a fork name to its corresponding `ForkName` enum variant.
impl TryFrom<String> for ForkName {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

/// Represents an inconsistency between the expected and actual fork names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InconsistentFork {
    /// The fork name expected at a certain slot.
    pub fork_at_slot: ForkName,
    /// The actual fork name found in the object.
    pub object_fork: ForkName,
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn previous_and_next_fork_consistent() {
        assert_eq!(ForkName::latest().next_fork(), None);
        assert_eq!(ForkName::Base.previous_fork(), None);

        for (prev_fork, fork) in ForkName::list_all().into_iter().tuple_windows() {
            assert_eq!(prev_fork.next_fork(), Some(fork));
            assert_eq!(fork.previous_fork(), Some(prev_fork));
        }
    }

    #[test]
    fn fork_name_case_insensitive_match() {
        assert_eq!(ForkName::from_str("BASE"), Ok(ForkName::Base));
        assert_eq!(ForkName::from_str("BaSe"), Ok(ForkName::Base));
        assert_eq!(ForkName::from_str("base"), Ok(ForkName::Base));

        assert_eq!(ForkName::from_str("PHASE0"), Ok(ForkName::Base));
        assert_eq!(ForkName::from_str("PhAsE0"), Ok(ForkName::Base));
        assert_eq!(ForkName::from_str("phase0"), Ok(ForkName::Base));

        assert_eq!(ForkName::from_str("ALTAIR"), Ok(ForkName::Altair));
        assert_eq!(ForkName::from_str("AlTaIr"), Ok(ForkName::Altair));
        assert_eq!(ForkName::from_str("altair"), Ok(ForkName::Altair));

        assert!(ForkName::from_str("NO_NAME").is_err());
        assert!(ForkName::from_str("no_name").is_err());
    }

    #[test]
    fn fork_name_bellatrix_or_merge() {
        assert_eq!(ForkName::from_str("bellatrix"), Ok(ForkName::Bellatrix));
        assert_eq!(ForkName::from_str("merge"), Ok(ForkName::Bellatrix));
        assert_eq!(ForkName::Bellatrix.to_string(), "bellatrix");
    }

    #[test]
    fn fork_name_latest() {
        assert_eq!(ForkName::latest(), *ForkName::list_all().last().unwrap());

        let mut fork = ForkName::Base;
        while let Some(next_fork) = fork.next_fork() {
            fork = next_fork;
        }
        assert_eq!(ForkName::latest(), fork);
    }

    #[test]
    fn fork_ord_consistent() {
        for (prev_fork, fork) in ForkName::list_all().into_iter().tuple_windows() {
            assert_eq!(prev_fork.next_fork(), Some(fork));
            assert_eq!(fork.previous_fork(), Some(prev_fork));
            assert!(prev_fork < fork);
        }
    }
}
