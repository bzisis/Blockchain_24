use crate::{test_utils::TestRandom, *};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Type alias for a transaction, represented as a variable list of bytes.
pub type Transaction<N> = VariableList<u8, N>;

/// Type alias for transactions, represented as a variable list of `Transaction`s.
pub type Transactions<E> =
    VariableList<Transaction<<E as EthSpec>::MaxBytesPerTransaction>, <E as EthSpec>::MaxTransactionsPerPayload>;

/// Type alias for withdrawals, represented as a variable list of `Withdrawal`.
pub type Withdrawals<E> = VariableList<Withdrawal, <E as EthSpec>::MaxWithdrawalsPerPayload>;

/// Represents an execution payload for different Ethereum execution layer variants.
///
/// This struct is used to encapsulate data related to Ethereum execution payloads, supporting
/// different variants such as Bellatrix, Capella, Deneb, and Electra. Each variant can have
/// different fields and behaviors, handled by the #[superstruct] attribute macro.
#[superstruct(
    variants(Bellatrix, Capella, Deneb, Electra),
    variant_attributes(
        derive(
            Default,
            Debug,
            Clone,
            Serialize,
            Deserialize,
            Encode,
            Decode,
            TreeHash,
            TestRandom,
            Derivative,
            arbitrary::Arbitrary
        ),
        derivative(PartialEq, Hash(bound = "E: EthSpec")),
        serde(bound = "E: EthSpec", deny_unknown_fields),
        arbitrary(bound = "E: EthSpec")
    ),
    cast_error(ty = "Error", expr = "BeaconStateError::IncorrectStateVariant"),
    partial_getter_error(ty = "Error", expr = "BeaconStateError::IncorrectStateVariant"),
    map_into(FullPayload, BlindedPayload),
    map_ref_into(ExecutionPayloadHeader)
)]
#[derive(
    Debug, Clone, Serialize, Encode, Deserialize, TreeHash, Derivative, arbitrary::Arbitrary,
)]
#[derivative(PartialEq, Hash(bound = "E: EthSpec"))]
#[serde(bound = "E: EthSpec", untagged)]
#[arbitrary(bound = "E: EthSpec")]
#[ssz(enum_behaviour = "transparent")]
#[tree_hash(enum_behaviour = "transparent")]
pub struct ExecutionPayload<E: EthSpec> {
    /// Parent hash of the execution block.
    #[superstruct(getter(copy))]
    pub parent_hash: ExecutionBlockHash,

    /// Address of the fee recipient for the execution payload.
    #[superstruct(getter(copy))]
    pub fee_recipient: Address,

    /// State root of the execution payload.
    #[superstruct(getter(copy))]
    pub state_root: Hash256,

    /// Receipts root of the execution payload.
    #[superstruct(getter(copy))]
    pub receipts_root: Hash256,

    /// Logs bloom filter of the execution payload.
    #[serde(with = "ssz_types::serde_utils::hex_fixed_vec")]
    pub logs_bloom: FixedVector<u8, E::BytesPerLogsBloom>,

    /// Previous RANDAO value of the execution payload.
    #[superstruct(getter(copy))]
    pub prev_randao: Hash256,

    /// Block number of the execution payload.
    #[serde(with = "serde_utils::quoted_u64")]
    #[superstruct(getter(copy))]
    pub block_number: u64,

    /// Gas limit of the execution payload.
    #[serde(with = "serde_utils::quoted_u64")]
    #[superstruct(getter(copy))]
    pub gas_limit: u64,

    /// Gas used by the execution payload.
    #[serde(with = "serde_utils::quoted_u64")]
    #[superstruct(getter(copy))]
    pub gas_used: u64,

    /// Timestamp of the execution payload.
    #[serde(with = "serde_utils::quoted_u64")]
    #[superstruct(getter(copy))]
    pub timestamp: u64,

    /// Extra data associated with the execution payload.
    #[serde(with = "ssz_types::serde_utils::hex_var_list")]
    pub extra_data: VariableList<u8, E::MaxExtraDataBytes>,

    /// Base fee per gas of the execution payload.
    #[serde(with = "serde_utils::quoted_u256")]
    #[superstruct(getter(copy))]
    pub base_fee_per_gas: Uint256,

    /// Block hash of the execution block.
    #[superstruct(getter(copy))]
    pub block_hash: ExecutionBlockHash,

    /// Transactions included in the execution payload.
    pub transactions: Transactions<E>,

    /// Withdrawals included in the execution payload.
    #[superstruct(only(Capella, Deneb, Electra))]
    pub withdrawals: Withdrawals<E>,

    /// Blob gas used in the execution payload (Capella, Deneb, Electra variants only).
    #[superstruct(only(Deneb, Electra), partial_getter(copy))]
    #[serde(with = "serde_utils::quoted_u64")]
    pub blob_gas_used: u64,

    /// Excess blob gas in the execution payload (Capella, Deneb, Electra variants only).
    #[superstruct(only(Deneb, Electra), partial_getter(copy))]
    #[serde(with = "serde_utils::quoted_u64")]
    pub excess_blob_gas: u64,

    /// Deposit receipts included in the execution payload (Electra variant only).
    #[superstruct(only(Electra))]
    pub deposit_receipts: VariableList<DepositReceipt, E::MaxDepositReceiptsPerPayload>,

    /// Withdrawal requests included in the execution payload (Electra variant only).
    #[superstruct(only(Electra))]
    pub withdrawal_requests: VariableList<ExecutionLayerWithdrawalRequest, E::MaxWithdrawalRequestsPerPayload>,
}

impl<'a, E: EthSpec> ExecutionPayloadRef<'a, E> {
    /// Clones the execution payload from a reference.
    ///
    /// This method emulates clone behavior on a normal reference type.
    pub fn clone_from_ref(&self) -> ExecutionPayload<E> {
        map_execution_payload_ref!(&'a _, self, move |payload, cons| {
            cons(payload);
            payload.clone().into()
        })
    }
}

impl<E: EthSpec> ExecutionPayload<E> {
    /// Constructs an execution payload from SSZ-encoded bytes.
    ///
    /// This method decodes SSZ-encoded bytes into the corresponding execution payload variant
    /// based on the provided fork name.
    ///
    /// # Arguments
    ///
    /// * `bytes` - SSZ-encoded bytes representing the execution payload.
    /// * `fork_name` - Fork name indicating the variant of the execution payload.
    ///
    /// # Returns
    ///
    /// A Result containing the decoded ExecutionPayload or a DecodeError if decoding fails.
    pub fn from_ssz_bytes(bytes: &[u8], fork_name: ForkName) -> Result<Self, ssz::DecodeError> {
        match fork_name {
            ForkName::Base | ForkName::Altair => Err(ssz::DecodeError::BytesInvalid(format!(
                "unsupported fork for ExecutionPayload: {fork_name}",
            ))),
            ForkName::Bellatrix => {
                ExecutionPayloadBellatrix::from_ssz_bytes(bytes).map(Self::Bellatrix)
            }
            ForkName::Capella => ExecutionPayloadCapella::from_ssz_bytes(bytes).map(Self::Capella),
            ForkName::Deneb => ExecutionPayloadDeneb::from_ssz_bytes(bytes).map(Self::Deneb),
            ForkName::Electra => Self::Electra(ExecutionPayloadElectra::from_ssz_bytes(bytes)?),
        }
    }

    /// Returns the maximum size of an execution payload for the Bellatrix variant.
    pub fn max_execution_payload_bellatrix_size() -> usize {
        // Fixed part
        ExecutionPayloadBellatrix::<E>::default().as_ssz_bytes().len()
            // Max size of variable length `extra_data` field
            + (E::max_extra_data_bytes() * <u8 as Encode>::ssz_fixed_len())
            // Max size of variable length `transactions` field
            + (E::max_transactions_per_payload() * (ssz::BYTES_PER_LENGTH_OFFSET + E::max_bytes_per_transaction()))
    }

    /// Returns the maximum size of an execution payload for the Capella variant.
    pub fn max_execution_payload_capella_size() -> usize {
        // Fixed part
        ExecutionPayloadCapella::<E>::default().as_ssz_bytes().len()
            // Max size of variable length `extra_data` field
            + (E::max_extra_data_bytes() * <u8 as Encode>::ssz_fixed_len())
            // Max size of variable length `transactions` field
            + (E::max_transactions_per_payload() * (ssz::BYTES_PER_LENGTH_OFFSET + E::max_bytes_per_transaction()))
            // Max size of variable length `withdrawals` field
            + (E::max_withdrawals_per_payload() * <Withdrawal as Encode>::ssz_fixed_len())
    }

    /// Returns the maximum size of an execution payload for the Deneb variant.
    pub fn max_execution_payload_deneb_size() -> usize {
        // Fixed part
        ExecutionPayloadDeneb::<E>::default().as_ssz_bytes().len()
            // Max size of variable length `extra_data` field
            + (E::max_extra_data_bytes() * <u8 as Encode>::ssz_fixed_len())
            // Max size of variable length `transactions` field
            + (E::max_transactions_per_payload() * (ssz::BYTES_PER_LENGTH_OFFSET + E::max_bytes_per_transaction()))
            // Max size of variable length `withdrawals` field
            + (E::max_withdrawals_per_payload() * <Withdrawal as Encode>::ssz_fixed_len())
    }

    /// Returns the maximum size of an execution payload for the Electra variant.
    pub fn max_execution_payload_electra_size() -> usize {
        // Fixed part
        ExecutionPayloadElectra::<E>::default().as_ssz_bytes().len()
            // Max size of variable length `extra_data` field
            + (E::max_extra_data_bytes() * <u8 as Encode>::ssz_fixed_len())
            // Max size of variable length `transactions` field
            + (E::max_transactions_per_payload() * (ssz::BYTES_PER_LENGTH_OFFSET + E::max_bytes_per_transaction()))
            // Max size of variable length `withdrawals` field
            + (E::max_withdrawals_per_payload() * <Withdrawal as Encode>::ssz_fixed_len())
    }
}

/// Trait for deserializing execution payloads based on the Ethereum fork version.
///
/// This trait provides a method to deserialize execution payloads from JSON based on the fork name.
impl<E: EthSpec> ForkVersionDeserialize for ExecutionPayload<E> {
    fn deserialize_by_fork<'de, D: serde::Deserializer<'de>>(
        value: serde_json::value::Value,
        fork_name: ForkName,
    ) -> Result<Self, D::Error> {
        let convert_err = |e| {
            serde::de::Error::custom(format!("ExecutionPayload failed to deserialize: {:?}", e))
        };

        Ok(match fork_name {
            ForkName::Bellatrix => {
                Self::Bellatrix(serde_json::from_value(value).map_err(convert_err)?)
            }
            ForkName::Capella => Self::Capella(serde_json::from_value(value).map_err(convert_err)?),
            ForkName::Deneb => Self::Deneb(serde_json::from_value(value).map_err(convert_err)?),
            ForkName::Electra => Self::Electra(serde_json::from_value(value).map_err(convert_err)?),
            ForkName::Base | ForkName::Altair => {
                return Err(serde::de::Error::custom(format!(
                    "ExecutionPayload failed to deserialize: unsupported fork '{}'",
                    fork_name
                )));
            }
        })
    }
}

impl<E: EthSpec> ExecutionPayload<E> {
    /// Returns the fork name associated with the execution payload variant.
    pub fn fork_name(&self) -> ForkName {
        match self {
            ExecutionPayload::Bellatrix(_) => ForkName::Bellatrix,
            ExecutionPayload::Capella(_) => ForkName::Capella,
            ExecutionPayload::Deneb(_) => ForkName::Deneb,
            ExecutionPayload::Electra(_) => ForkName::Electra,
        }
    }
}
