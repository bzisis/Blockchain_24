//! Payload related types

//! Optimism builder support

use alloy_rlp::Encodable; // Importing Encodable trait from alloy_rlp
use reth_chainspec::{ChainSpec, EthereumHardforks}; // Importing chain specifications and Ethereum hardforks from reth_chainspec
use reth_evm_optimism::revm_spec_by_timestamp_after_bedrock; // Importing REVM specification based on timestamp after bedrock from reth_evm_optimism
use reth_payload_builder::EthPayloadBuilderAttributes; // Importing Ethereum payload builder attributes from reth_payload_builder
use reth_payload_primitives::{BuiltPayload, PayloadBuilderAttributes}; // Importing built payload and payload builder attributes from reth_payload_primitives
use reth_primitives::{
    revm_primitives::{BlobExcessGasAndPrice, BlockEnv, CfgEnv, CfgEnvWithHandlerCfg, SpecId}, // Importing REVM primitives like BlobExcessGasAndPrice, BlockEnv, etc.
    Address, BlobTransactionSidecar, Header, SealedBlock, TransactionSigned, Withdrawals, B256, // Importing various primitives like Address, Header, SealedBlock, etc.
    U256, // Importing U256 type
};
use reth_rpc_types::engine::{ // Importing engine related types from reth_rpc_types
    ExecutionPayloadEnvelopeV2, ExecutionPayloadV1, OptimismExecutionPayloadEnvelopeV3, // Importing different execution payload envelopes
    OptimismExecutionPayloadEnvelopeV4, PayloadId, // Importing Optimism specific payload envelopes and PayloadId
};
use reth_rpc_types_compat::engine::payload::{ // Importing payload related compatibility types
    block_to_payload_v1, block_to_payload_v3, block_to_payload_v4, // Importing functions to convert block to payload for different versions
    convert_block_to_payload_field_v2, // Importing function to convert block to payload field for version 2
};
use revm::primitives::HandlerCfg; // Importing HandlerCfg from revm primitives
use std::sync::Arc; // Importing Arc for atomic reference counting

/// Re-export for use in downstream arguments.
pub use reth_rpc_types::engine::OptimismPayloadAttributes; // Re-exporting OptimismPayloadAttributes from reth_rpc_types

/// Optimism Payload Builder Attributes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimismPayloadBuilderAttributes {
    /// Inner ethereum payload builder attributes
    pub payload_attributes: EthPayloadBuilderAttributes, // Ethereum payload builder attributes
    /// `NoTxPool` option for the generated payload
    pub no_tx_pool: bool, // Flag indicating absence of transaction pool
    /// Transactions for the generated payload
    pub transactions: Vec<TransactionSigned>, // Vector of signed transactions
    /// The gas limit for the generated payload
    pub gas_limit: Option<u64>, // Optional gas limit for the payload
}

impl PayloadBuilderAttributes for OptimismPayloadBuilderAttributes {
    type RpcPayloadAttributes = OptimismPayloadAttributes; // Associated type for RPC payload attributes
    type Error = alloy_rlp::Error; // Error type for encoding

    /// Creates a new payload builder for the given parent block and the attributes.
    ///
    /// Derives the unique [`PayloadId`] for the given parent and attributes
    fn try_new(parent: B256, attributes: OptimismPayloadAttributes) -> Result<Self, Self::Error> {
        let (id, transactions) = {
            let transactions: Vec<_> = attributes
                .transactions
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(|tx| TransactionSigned::decode_enveloped(&mut tx.as_ref()))
                .collect::<Result<_, _>>()?;
            (payload_id_optimism(&parent, &attributes, &transactions), transactions)
        };

        let payload_attributes = EthPayloadBuilderAttributes {
            id,
            parent,
            timestamp: attributes.payload_attributes.timestamp,
            suggested_fee_recipient: attributes.payload_attributes.suggested_fee_recipient,
            prev_randao: attributes.payload_attributes.prev_randao,
            withdrawals: attributes.payload_attributes.withdrawals.unwrap_or_default().into(),
            parent_beacon_block_root: attributes.payload_attributes.parent_beacon_block_root,
        };

        Ok(Self {
            payload_attributes,
            no_tx_pool: attributes.no_tx_pool.unwrap_or_default(),
            transactions,
            gas_limit: attributes.gas_limit,
        })
    }

    /// Returns the unique payload ID associated with this payload builder.
    fn payload_id(&self) -> PayloadId {
        self.payload_attributes.id
    }

    /// Returns the parent block hash associated with this payload builder.
    fn parent(&self) -> B256 {
        self.payload_attributes.parent
    }

    /// Returns the timestamp associated with this payload builder.
    fn timestamp(&self) -> u64 {
        self.payload_attributes.timestamp
    }

    /// Returns the parent beacon block root associated with this payload builder.
    fn parent_beacon_block_root(&self) -> Option<B256> {
        self.payload_attributes.parent_beacon_block_root
    }

    /// Returns the suggested fee recipient associated with this payload builder.
    fn suggested_fee_recipient(&self) -> Address {
        self.payload_attributes.suggested_fee_recipient
    }

    /// Returns the previous RANDAO associated with this payload builder.
    fn prev_randao(&self) -> B256 {
        self.payload_attributes.prev_randao
    }

    /// Returns the withdrawals associated with this payload builder.
    fn withdrawals(&self) -> &Withdrawals {
        &self.payload_attributes.withdrawals
    }

    /// Returns the configured EVM environment and block environment.
    ///
    /// This includes setting the chain ID and calculating base fee and blob excess gas based on the parent block.
    fn cfg_and_block_env(
        &self,
        chain_spec: &ChainSpec,
        parent: &Header,
    ) -> (CfgEnvWithHandlerCfg, BlockEnv) {
        // configure evm env based on parent block
        let cfg = CfgEnv::default().with_chain_id(chain_spec.chain().id());

        // ensure we're not missing any timestamp based hardforks
        let spec_id = revm_spec_by_timestamp_after_bedrock(chain_spec, self.timestamp());

        // if the parent block did not have excess blob gas (i.e. it was pre-cancun), but it is
        // cancun now, we need to set the excess blob gas to the default value
        let blob_excess_gas_and_price = parent
            .next_block_excess_blob_gas()
            .or_else(|| {
                if spec_id.is_enabled_in(SpecId::CANCUN) {
                    // default excess blob gas is zero
                    Some(0)
                } else {
                    None
                }
            })
            .map(BlobExcessGasAndPrice::new);

        let block_env = BlockEnv {
            number: U256::from(parent.number + 1),
            coinbase: self.suggested_fee_recipient(),
            timestamp: U256::from(self.timestamp()),
            difficulty: U256::ZERO,
            prevrandao: Some(self.prev_randao()),
            gas_limit: U256::from(parent.gas_limit),
            // calculate basefee based on parent block's gas usage
            basefee: U256::from(
                parent
                    .next_block_base_fee(chain_spec.base_fee_params_at_timestamp(self.timestamp()))
                    .unwrap_or_default(),
            ),
            // calculate excess gas based on parent block's blob gas usage
            blob_excess_gas_and_price,
        };

        let cfg_with_handler_cfg;
        {
            cfg_with_handler_cfg = CfgEnvWithHandlerCfg {
                cfg_env: cfg,
                handler_cfg: HandlerCfg { spec_id, is_optimism: chain_spec.is_optimism() },
            };
        }

        (cfg_with_handler_cfg, block_env)
    }
}

/// Contains the built payload.
#[derive(Debug, Clone)]
pub struct OptimismBuiltPayload {
    /// Identifier of the payload
    pub(crate) id: PayloadId,
    /// The built block
    pub(crate) block: SealedBlock,
    /// The fees of the block
    pub(crate) fees: U256,
    /// The blobs, proofs, and commitments in the block. If the block is pre-cancun, this will be
    /// empty.
    pub(crate) sidecars: Vec<BlobTransactionSidecar>,
    /// The rollup's chainspec.
    pub(crate) chain_spec: Arc<ChainSpec>,
    /// The payload attributes.
    pub(crate) attributes: OptimismPayloadBuilderAttributes,
}

// === impl BuiltPayload ===

impl OptimismBuiltPayload {
    /// Initializes the payload with the given initial block.
    pub const fn new(
        id: PayloadId,
        block: SealedBlock,
        fees: U256,
        chain_spec: Arc<ChainSpec>,
        attributes: OptimismPayloadBuilderAttributes,
    ) -> Self {
        Self { id, block, fees, sidecars: Vec::new(), chain_spec, attributes }
    }

    /// Returns the identifier of the payload.
    pub const fn id(&self) -> PayloadId {
        self.id
    }

    /// Returns the built block(sealed)
    pub const fn block(&self) -> &SealedBlock {
        &self.block
    }

    /// Fees of the block
    pub const fn fees(&self) -> U256 {
        self.fees
    }

    /// Adds sidecars to the payload.
    pub fn extend_sidecars(&mut self, sidecars: Vec<BlobTransactionSidecar>) {
        self.sidecars.extend(sidecars)
    }
}

impl BuiltPayload for OptimismBuiltPayload {
    fn block(&self) -> &SealedBlock {
        &self.block
    }

    fn fees(&self) -> U256 {
        self.fees
    }
}

impl<'a> BuiltPayload for &'a OptimismBuiltPayload {
    fn block(&self) -> &SealedBlock {
        (**self).block()
    }

    fn fees(&self) -> U256 {
        (**self).fees()
    }
}

// V1 engine_getPayloadV1 response
impl From<OptimismBuiltPayload> for ExecutionPayloadV1 {
    fn from(value: OptimismBuiltPayload) -> Self {
        block_to_payload_v1(value.block)
    }
}

// V2 engine_getPayloadV2 response
impl From<OptimismBuiltPayload> for ExecutionPayloadEnvelopeV2 {
    fn from(value: OptimismBuiltPayload) -> Self {
        let OptimismBuiltPayload { block, fees, .. } = value;

        Self { block_value: fees, execution_payload: convert_block_to_payload_field_v2(block) }
    }
}

impl From<OptimismBuiltPayload> for OptimismExecutionPayloadEnvelopeV3 {
    fn from(value: OptimismBuiltPayload) -> Self {
        let OptimismBuiltPayload { block, fees, sidecars, chain_spec, attributes, .. } = value;

        let parent_beacon_block_root =
            if chain_spec.is_cancun_active_at_timestamp(attributes.timestamp()) {
                attributes.parent_beacon_block_root().unwrap_or(B256::ZERO)
            } else {
                B256::ZERO
            };
        Self {
            execution_payload: block_to_payload_v3(block).0,
            block_value: fees,
            // From the engine API spec:
            //
            // > Client software **MAY** use any heuristics to decide whether to set
            // `shouldOverrideBuilder` flag or not. If client software does not implement any
            // heuristic this flag **SHOULD** be set to `false`.
            //
            // Spec:
            // <https://github.com/ethereum/execution-apis/blob/fe8e13c288c592ec154ce25c534e26cb7ce0530d/src/engine/cancun.md#specification-2>
            should_override_builder: false,
            blobs_bundle: sidecars.into_iter().map(Into::into).collect::<Vec<_>>().into(),
            parent_beacon_block_root,
        }
    }
}

impl From<OptimismBuiltPayload> for OptimismExecutionPayloadEnvelopeV4 {
    fn from(value: OptimismBuiltPayload) -> Self {
        let OptimismBuiltPayload { block, fees, sidecars, chain_spec, attributes, .. } = value;

        let parent_beacon_block_root =
            if chain_spec.is_cancun_active_at_timestamp(attributes.timestamp()) {
                attributes.parent_beacon_block_root().unwrap_or(B256::ZERO)
            } else {
                B256::ZERO
            };
        Self {
            execution_payload: block_to_payload_v4(block),
            block_value: fees,
            // From the engine API spec:
            //
            // > Client software **MAY** use any heuristics to decide whether to set
            // `shouldOverrideBuilder` flag or not. If client software does not implement any
            // heuristic this flag **SHOULD** be set to `false`.
            //
            // Spec:
            // <https://github.com/ethereum/execution-apis/blob/fe8e13c288c592ec154ce25c534e26cb7ce0530d/src/engine/cancun.md#specification-2>
            should_override_builder: false,
            blobs_bundle: sidecars.into_iter().map(Into::into).collect::<Vec<_>>().into(),
            parent_beacon_block_root,
        }
    }
}

/// Generates the payload id for the configured payload from the [`OptimismPayloadAttributes`].
///
/// Returns an 8-byte identifier by hashing the payload components with sha256 hash.
pub(crate) fn payload_id_optimism(
    parent: &B256,
    attributes: &OptimismPayloadAttributes,
    txs: &[TransactionSigned],
) -> PayloadId {
    use sha2::{Digest, Sha256};
    use std::convert::TryInto;

    // Create a SHA-256 hasher instance
    let mut hasher = Sha256::new();

    // Update the hasher with parent hash
    hasher.update(parent.as_slice());

    // Update the hasher with timestamp in big endian bytes
    hasher.update(&attributes.payload_attributes.timestamp.to_be_bytes());

    // Update the hasher with previous RANDAO commitment
    hasher.update(attributes.payload_attributes.prev_randao.as_slice());

    // Update the hasher with suggested fee recipient
    hasher.update(attributes.payload_attributes.suggested_fee_recipient.as_slice());

    // If there are withdrawals, encode and hash them
    if let Some(withdrawals) = &attributes.payload_attributes.withdrawals {
        let mut buf = Vec::new();
        withdrawals.encode(&mut buf).expect("failed to encode withdrawals");
        hasher.update(&buf);
    }

    // Update the hasher with parent beacon block root if present
    if let Some(parent_beacon_block) = attributes.payload_attributes.parent_beacon_block_root {
        hasher.update(parent_beacon_block);
    }

    // Determine if no transaction pool flag is set
    let no_tx_pool = attributes.no_tx_pool.unwrap_or_default();

    // Update the hasher with no_tx_pool flag
    hasher.update([no_tx_pool as u8]);

    // If no transaction pool or transactions exist, update hasher with transaction count and hashes
    if no_tx_pool || !txs.is_empty() {
        hasher.update(txs.len().to_be_bytes());
        for tx in txs {
            hasher.update(tx.hash());
        }
    }

    // If gas limit is specified, update hasher with gas limit in big endian bytes
    if let Some(gas_limit) = attributes.gas_limit {
        hasher.update(gas_limit.to_be_bytes());
    }

    // Obtain the final hash result from the hasher
    let hash_result = hasher.finalize();

    // Extract the first 8 bytes as the payload id
    let payload_id_bytes = &hash_result[..8];
    let payload_id_array: [u8; 8] = payload_id_bytes.try_into().expect("expected 8 bytes");

    // Return the constructed PayloadId
    PayloadId::new(payload_id_array)
}
