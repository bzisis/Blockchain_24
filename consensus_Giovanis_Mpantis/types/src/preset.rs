use crate::{ChainSpec, Epoch, EthSpec, Unsigned};
use serde::{Deserialize, Serialize};

/// Value-level representation of an Ethereum consensus "preset".
///
/// This struct represents configurable parameters that define the behavior of the Ethereum
/// consensus protocol. Each preset corresponds to a specific set of constants and configurations
/// used in different phases of Ethereum's evolution.
///
/// This should only be used to check consistency of the compile-time constants
/// with a preset YAML file, or to make preset values available to the API. Prefer
/// the constants on `EthSpec` or the fields on `ChainSpec` to constructing and using
/// one of these structs.
///
/// For more details, refer to:
/// [Ethereum 2.0 Specs Phase 0 Preset](https://github.com/ethereum/eth2.0-specs/blob/dev/presets/mainnet/phase0.yaml)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BasePreset {
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of committees per slot.
    pub max_committees_per_slot: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Target size of each committee.
    pub target_committee_size: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of validators per committee.
    pub max_validators_per_committee: u64,
    #[serde(with = "serde_utils::quoted_u8")]
    /// Number of rounds for the shuffle.
    pub shuffle_round_count: u8,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Quotient for the hysteresis for adjustments to the justification distance.
    pub hysteresis_quotient: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Multiplier for decreasing the justified checkpoint distance.
    pub hysteresis_downward_multiplier: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Multiplier for increasing the justified checkpoint distance.
    pub hysteresis_upward_multiplier: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Number of safe slots before updating justified checkpoint.
    pub safe_slots_to_update_justified: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum deposit amount required for validators.
    pub min_deposit_amount: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum effective balance per validator.
    pub max_effective_balance: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Increment for effective balance.
    pub effective_balance_increment: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Delay in epochs before attestations can be included in a block.
    pub min_attestation_inclusion_delay: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Number of slots in each epoch.
    pub slots_per_epoch: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum epoch lookahead for seed selection.
    pub min_seed_lookahead: Epoch,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum epoch lookahead for seed selection.
    pub max_seed_lookahead: Epoch,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Number of epochs in each eth1 voting period.
    pub epochs_per_eth1_voting_period: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Number of slots per historical root stored in state.
    pub slots_per_historical_root: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum epochs before inactivity penalty is applied.
    pub min_epochs_to_inactivity_penalty: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Number of epochs in each historical vector.
    pub epochs_per_historical_vector: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Number of epochs in each slashings vector.
    pub epochs_per_slashings_vector: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of historical roots stored in state.
    pub historical_roots_limit: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of validators allowed.
    pub validator_registry_limit: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Base factor for reward computation.
    pub base_reward_factor: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Quotient for whistleblower reward calculation.
    pub whistleblower_reward_quotient: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Quotient for proposer reward calculation.
    pub proposer_reward_quotient: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Quotient for inactivity penalty calculation.
    pub inactivity_penalty_quotient: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum slashing penalty quotient.
    pub min_slashing_penalty_quotient: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Multiplier for proportional slashing penalties.
    pub proportional_slashing_multiplier: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of proposer slashings per block.
    pub max_proposer_slashings: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of attester slashings per block.
    pub max_attester_slashings: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of attestations per block.
    pub max_attestations: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of deposits per block.
    pub max_deposits: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum number of voluntary exits per block.
    pub max_voluntary_exits: u64,
}

impl BasePreset {
    /// Creates a `BasePreset` from the provided `ChainSpec`.
    ///
    /// Converts the constants from `ChainSpec` and `EthSpec` into a `BasePreset`.
    pub fn from_chain_spec<E: EthSpec>(spec: &ChainSpec) -> Self {
        Self {
            max_committees_per_slot: spec.max_committees_per_slot as u64,
            target_committee_size: spec.target_committee_size as u64,
            max_validators_per_committee: E::MaxValidatorsPerCommittee::to_u64(),
            shuffle_round_count: spec.shuffle_round_count,
            hysteresis_quotient: spec.hysteresis_quotient,
            hysteresis_downward_multiplier: spec.hysteresis_downward_multiplier,
            hysteresis_upward_multiplier: spec.hysteresis_upward_multiplier,
            safe_slots_to_update_justified: spec.safe_slots_to_update_justified,
            min_deposit_amount: spec.min_deposit_amount,
            max_effective_balance: spec.max_effective_balance,
            effective_balance_increment: spec.effective_balance_increment,
            min_attestation_inclusion_delay: spec.min_attestation_inclusion_delay,
            slots_per_epoch: E::SlotsPerEpoch::to_u64(),
            min_seed_lookahead: spec.min_seed_lookahead,
            max_seed_lookahead: spec.max_seed_lookahead,
            epochs_per_eth1_voting_period: E::EpochsPerEth1VotingPeriod::to_u64(),
            slots_per_historical_root: E::SlotsPerHistoricalRoot::to_u64(),
            min_epochs_to_inactivity_penalty: spec.min_epochs_to_inactivity_penalty,
            epochs_per_historical_vector: E::EpochsPerHistoricalVector::to_u64(),
            epochs_per_slashings_vector: E::EpochsPerSlashingsVector::to_u64(),
            historical_roots_limit: E::HistoricalRootsLimit::to_u64(),
            validator_registry_limit: E::ValidatorRegistryLimit::to_u64(),
            base_reward_factor: spec.base_reward_factor,
            whistleblower_reward_quotient: spec.whistleblower_reward_quotient,
            proposer_reward_quotient: spec.proposer_reward_quotient,
            inactivity_penalty_quotient: spec.inactivity_penalty_quotient,
            min_slashing_penalty_quotient: spec.min_slashing_penalty_quotient,
            proportional_slashing_multiplier: spec.proportional_slashing_multiplier,
            max_proposer_slashings: E::MaxProposerSlashings::to_u64(),
            max_attester_slashings: E::MaxAttesterSlashings::to_u64(),
            max_attestations: E::MaxAttestations::to_u64(),
            max_deposits: E::MaxDeposits::to_u64(),
            max_voluntary_exits: E::MaxVoluntaryExits::to_u64(),
        }
    }
}

/// Value-level representation of an Altair Ethereum consensus "preset".
///
/// This struct extends `BasePreset` with additional parameters specific to the Altair upgrade.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct AltairPreset {
    #[serde(with = "serde_utils::quoted_u64")]
    /// Quotient for the inactivity penalty calculation in Altair.
    pub inactivity_penalty_quotient_altair: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum slashing penalty quotient in Altair.
    pub min_slashing_penalty_quotient_altair: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Multiplier for proportional slashing penalties in Altair.
    pub proportional_slashing_multiplier_altair: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Size of the sync committee in Altair.
    pub sync_committee_size: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Number of epochs in each sync committee period in Altair.
    pub epochs_per_sync_committee_period: Epoch,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum participants required in the sync committee in Altair.
    pub min_sync_committee_participants: u64,
}

impl AltairPreset {
    /// Creates an `AltairPreset` from the provided `ChainSpec`.
    ///
    /// Converts the constants from `ChainSpec` and `EthSpec` into an `AltairPreset`.
    pub fn from_chain_spec<E: EthSpec>(spec: &ChainSpec) -> Self {
        Self {
            inactivity_penalty_quotient_altair: spec.inactivity_penalty_quotient_altair,
            min_slashing_penalty_quotient_altair: spec.min_slashing_penalty_quotient_altair,
            proportional_slashing_multiplier_altair: spec.proportional_slashing_multiplier_altair,
            sync_committee_size: E::SyncCommitteeSize::to_u64(),
            epochs_per_sync_committee_period: spec.epochs_per_sync_committee_period,
            min_sync_committee_participants: spec.min_sync_committee_participants,
        }
    }
}

/// Value-level representation of a Bellatrix Ethereum consensus "preset".
///
/// This struct extends `BasePreset` with additional parameters specific to the Bellatrix upgrade.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BellatrixPreset {
    #[serde(with = "serde_utils::quoted_u64")]
    /// Quotient for the inactivity penalty calculation in Bellatrix.
    pub inactivity_penalty_quotient_bellatrix: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum slashing penalty quotient in Bellatrix.
    pub min_slashing_penalty_quotient_bellatrix: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Multiplier for proportional slashing penalties in Bellatrix.
    pub proportional_slashing_multiplier_bellatrix: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum bytes per transaction in Bellatrix.
    pub max_bytes_per_transaction: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum transactions per payload in Bellatrix.
    pub max_transactions_per_payload: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Bytes per logs bloom in Bellatrix.
    pub bytes_per_logs_bloom: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum extra data bytes in Bellatrix.
    pub max_extra_data_bytes: u64,
}

impl BellatrixPreset {
    /// Creates a `BellatrixPreset` from the provided `ChainSpec`.
    ///
    /// Converts the constants from `ChainSpec` and `EthSpec` into a `BellatrixPreset`.
    pub fn from_chain_spec<E: EthSpec>(spec: &ChainSpec) -> Self {
        Self {
            inactivity_penalty_quotient_bellatrix: spec.inactivity_penalty_quotient_bellatrix,
            min_slashing_penalty_quotient_bellatrix: spec.min_slashing_penalty_quotient_bellatrix,
            proportional_slashing_multiplier_bellatrix: spec.proportional_slashing_multiplier_bellatrix,
            max_bytes_per_transaction: E::max_bytes_per_transaction() as u64,
            max_transactions_per_payload: E::max_transactions_per_payload() as u64,
            bytes_per_logs_bloom: E::bytes_per_logs_bloom() as u64,
            max_extra_data_bytes: E::max_extra_data_bytes() as u64,
        }
    }
}

/// Value-level representation of a Capella Ethereum consensus "preset".
///
/// This struct extends `BasePreset` with additional parameters specific to the Capella upgrade.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct CapellaPreset {
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum BLS aggregation to execution changes in Capella.
    pub max_bls_to_execution_changes: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum withdrawals per payload in Capella.
    pub max_withdrawals_per_payload: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum validators per withdrawals sweep in Capella.
    pub max_validators_per_withdrawals_sweep: u64,
}

impl CapellaPreset {
    /// Creates a `CapellaPreset` from the provided `ChainSpec`.
    ///
    /// Converts the constants from `ChainSpec` and `EthSpec` into a `CapellaPreset`.
    pub fn from_chain_spec<E: EthSpec>(spec: &ChainSpec) -> Self {
        Self {
            max_bls_to_execution_changes: E::max_bls_to_execution_changes() as u64,
            max_withdrawals_per_payload: E::max_withdrawals_per_payload() as u64,
            max_validators_per_withdrawals_sweep: spec.max_validators_per_withdrawals_sweep,
        }
    }
}

/// Value-level representation of a Deneb Ethereum consensus "preset".
///
/// This struct extends `BasePreset` with additional parameters specific to the Deneb upgrade.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct DenebPreset {
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum blobs per block in Deneb.
    pub max_blobs_per_block: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum blob commitments per block in Deneb.
    pub max_blob_commitments_per_block: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Field elements per blob in Deneb.
    pub field_elements_per_blob: u64,
}

impl DenebPreset {
    /// Creates a `DenebPreset` from the provided `EthSpec`.
    ///
    /// Converts the constants from `EthSpec` into a `DenebPreset`.
    pub fn from_chain_spec<E: EthSpec>(_spec: &ChainSpec) -> Self {
        Self {
            max_blobs_per_block: E::max_blobs_per_block() as u64,
            max_blob_commitments_per_block: E::max_blob_commitments_per_block() as u64,
            field_elements_per_blob: E::field_elements_per_blob() as u64,
        }
    }
}

/// Value-level representation of an Electra Ethereum consensus "preset".
///
/// This struct extends `BasePreset` with additional parameters specific to the Electra upgrade.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct ElectraPreset {
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum activation balance for validators in Electra.
    pub min_activation_balance: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum effective balance per validator in Electra.
    pub max_effective_balance_electra: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Minimum slashing penalty quotient in Electra.
    pub min_slashing_penalty_quotient_electra: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Quotient for whistleblower reward calculation in Electra.
    pub whistleblower_reward_quotient_electra: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum pending partials per withdrawals sweep in Electra.
    pub max_pending_partials_per_withdrawals_sweep: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Pending balance deposits limit in Electra.
    pub pending_balance_deposits_limit: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Pending partial withdrawals limit in Electra.
    pub pending_partial_withdrawals_limit: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Pending consolidations limit in Electra.
    pub pending_consolidations_limit: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum consolidations per validator in Electra.
    pub max_consolidations: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum deposit receipts per payload in Electra.
    pub max_deposit_receipts_per_payload: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum attester slashings per block in Electra.
    pub max_attester_slashings_electra: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum attestations per block in Electra.
    pub max_attestations_electra: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    /// Maximum withdrawal requests per payload in Electra.
    pub max_withdrawal_requests_per_payload: u64,
}

impl ElectraPreset {
    /// Creates an `ElectraPreset` from the provided `ChainSpec`.
    ///
    /// Converts the constants from `ChainSpec` and `EthSpec` into an `ElectraPreset`.
    pub fn from_chain_spec<E: EthSpec>(spec: &ChainSpec) -> Self {
        Self {
            min_activation_balance: spec.min_activation_balance,
            max_effective_balance_electra: spec.max_effective_balance_electra,
            min_slashing_penalty_quotient_electra: spec.min_slashing_penalty_quotient_electra,
            whistleblower_reward_quotient_electra: spec.whistleblower_reward_quotient_electra,
            max_pending_partials_per_withdrawals_sweep: spec.max_pending_partials_per_withdrawals_sweep,
            pending_balance_deposits_limit: E::pending_balance_deposits_limit() as u64,
            pending_partial_withdrawals_limit: E::pending_partial_withdrawals_limit() as u64,
            pending_consolidations_limit: E::pending_consolidations_limit() as u64,
            max_consolidations: E::max_consolidations() as u64,
            max_deposit_receipts_per_payload: E::max_deposit_receipts_per_payload() as u64,
            max_attester_slashings_electra: E::max_attester_slashings_electra() as u64,
            max_attestations_electra: E::max_attestations_electra() as u64,
            max_withdrawal_requests_per_payload: E::max_withdrawal_requests_per_payload() as u64,
        }
    }
}

/// Unit tests for the Ethereum consensus presets.
#[cfg(test)]
mod test {
    use super::*;
    use crate::{GnosisEthSpec, MainnetEthSpec, MinimalEthSpec};
    use serde::de::DeserializeOwned;
    use std::env;
    use std::fs::File;
    use std::path::PathBuf;

    /// Returns the base path for preset files.
    fn presets_base_path() -> PathBuf {
        env::var("CARGO_MANIFEST_DIR")
            .expect("should know manifest dir")
            .parse::<PathBuf>()
            .expect("should parse manifest dir as path")
            .join("presets")
    }

    /// Loads a preset from file.
    fn preset_from_file<T: DeserializeOwned>(preset_name: &str, filename: &str) -> T {
        let f = File::open(presets_base_path().join(preset_name).join(filename))
            .expect("preset file exists");
        serde_yaml::from_reader(f).unwrap()
    }

    /// Tests the consistency of presets for a given `EthSpec`.
    fn preset_test<E: EthSpec>() {
        let preset_name = E::spec_name().to_string();
        let spec = E::default_spec();

        let phase0: BasePreset = preset_from_file(&preset_name, "phase0.yaml");
        assert_eq!(phase0, BasePreset::from_chain_spec::<E>(&spec));

        let altair: AltairPreset = preset_from_file(&preset_name, "altair.yaml");
        assert_eq!(altair, AltairPreset::from_chain_spec::<E>(&spec));

        let bellatrix: BellatrixPreset = preset_from_file(&preset_name, "bellatrix.yaml");
        assert_eq!(bellatrix, BellatrixPreset::from_chain_spec::<E>(&spec));

        let capella: CapellaPreset = preset_from_file(&preset_name, "capella.yaml");
        assert_eq!(capella, CapellaPreset::from_chain_spec::<E>(&spec));

        let deneb: DenebPreset = preset_from_file(&preset_name, "deneb.yaml");
        assert_eq!(deneb, DenebPreset::from_chain_spec::<E>(&spec));

        let electra: ElectraPreset = preset_from_file(&preset_name, "electra.yaml");
        assert_eq!(electra, ElectraPreset::from_chain_spec::<E>(&spec));
    }

    /// Tests for mainnet presets consistency.
    #[test]
    fn mainnet_presets_consistent() {
        preset_test::<MainnetEthSpec>();
    }

    /// Tests for gnosis presets consistency.
    #[test]
    fn gnosis_presets_consistent() {
        preset_test::<GnosisEthSpec>();
    }

    /// Tests for minimal presets consistency.
    #[test]
    fn minimal_presets_consistent() {
        preset_test::<MinimalEthSpec>();
    }
}
